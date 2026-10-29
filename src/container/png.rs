use image::{DynamicImage, GrayImage};
use std::fs::File;
use std::io::{BufWriter, Cursor, Read};
use std::path::Path;

use crate::core::error::{DvmaError, Result};
use crate::crypto::shamir::KeyShare;

/// PNG custom chunk name for Share B.
/// Must follow PNG naming conventions:
/// - 4 ASCII letters: [d, v, M, a]
/// - First byte 'd' (0x64): lowercase -> Ancillary chunk
/// - Second byte 'v' (0x76): lowercase -> Private chunk
/// - Third byte 'M' (0x4D): UPPERCASE -> Reserved by PNG spec
/// - Fourth byte 'a' (0x61): lowercase -> Safe-to-copy
pub const CUSTOM_CHUNK_NAME: [u8; 4] = *b"dvMa";

/// Saves a grayscale matrix image along with embedded Share B into a PNG file.
pub fn save_dvma_png<P: AsRef<Path>>(
    path: P,
    matrix_img: &GrayImage,
    share_b: &KeyShare,
) -> Result<()> {
    // 1. Encode matrix image to PNG buffer in memory using `image` crate
    let mut raw_png_bytes = Vec::new();
    let dynamic_img = DynamicImage::ImageLuma8(matrix_img.clone());
    dynamic_img
        .write_to(
            &mut Cursor::new(&mut raw_png_bytes),
            image::ImageFormat::Png,
        )
        .map_err(|e| {
            DvmaError::ContainerError(format!("Failed to write PNG matrix image: {}", e))
        })?;

    // 2. Parse PNG chunk stream and inject custom chunk `dvMa` before `IEND`
    let share_b_bytes = share_b.serialize();
    let modified_png_bytes =
        inject_custom_chunk(&raw_png_bytes, CUSTOM_CHUNK_NAME, &share_b_bytes)?;

    // 3. Write output to target path
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    use std::io::Write;
    writer.write_all(&modified_png_bytes)?;
    writer.flush()?;

    Ok(())
}

/// Reads a DVMA PNG file, returning the decoded matrix GrayImage and extracting Share B.
pub fn load_dvma_png<P: AsRef<Path>>(path: P) -> Result<(GrayImage, KeyShare)> {
    let mut file = File::open(path)?;
    let mut png_bytes = Vec::new();
    file.read_to_end(&mut png_bytes)?;

    // 1. Extract Share B from custom PNG chunk `dvMa`
    let share_b_data = extract_custom_chunk(&png_bytes, CUSTOM_CHUNK_NAME)?;
    let share_b = KeyShare::deserialize(&share_b_data)
        .map_err(|e| DvmaError::ContainerError(format!("Failed to deserialize Share B: {}", e)))?;

    // 2. Decode matrix image
    let dynamic_img = image::load_from_memory_with_format(&png_bytes, image::ImageFormat::Png)
        .map_err(|e| DvmaError::ContainerError(format!("Failed to parse PNG image: {}", e)))?;

    let gray_img = dynamic_img.into_luma8();

    Ok((gray_img, share_b))
}

fn inject_custom_chunk(
    png_bytes: &[u8],
    chunk_name: [u8; 4],
    chunk_data: &[u8],
) -> Result<Vec<u8>> {
    let png_header: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
    if png_bytes.len() < 8 || png_bytes[0..8] != png_header {
        return Err(DvmaError::ContainerError("Invalid PNG signature".into()));
    }

    let mut output = Vec::with_capacity(png_bytes.len() + chunk_data.len() + 12);
    output.extend_from_slice(&png_header);

    let mut cursor = 8;
    let mut injected = false;

    while cursor < png_bytes.len() {
        if cursor + 8 > png_bytes.len() {
            return Err(DvmaError::ContainerError(
                "Truncated PNG chunk header".into(),
            ));
        }

        let length = u32::from_be_bytes(png_bytes[cursor..cursor + 4].try_into().unwrap()) as usize;
        let name = &png_bytes[cursor + 4..cursor + 8];

        if name == b"IEND" && !injected {
            // Inject custom chunk before IEND
            let chunk_len = chunk_data.len() as u32;
            output.extend_from_slice(&chunk_len.to_be_bytes());
            output.extend_from_slice(&chunk_name);
            output.extend_from_slice(chunk_data);

            let mut hasher = crc32fast::Hasher::new();
            hasher.update(&chunk_name);
            hasher.update(chunk_data);
            let crc = hasher.finalize();
            output.extend_from_slice(&crc.to_be_bytes());

            injected = true;
        }

        let chunk_total = 12 + length;
        if cursor + chunk_total > png_bytes.len() {
            return Err(DvmaError::ContainerError("Truncated PNG chunk data".into()));
        }

        output.extend_from_slice(&png_bytes[cursor..cursor + chunk_total]);
        cursor += chunk_total;
    }

    if !injected {
        return Err(DvmaError::ContainerError("PNG missing IEND chunk".into()));
    }

    Ok(output)
}

fn extract_custom_chunk(png_bytes: &[u8], chunk_name: [u8; 4]) -> Result<Vec<u8>> {
    let png_header: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
    if png_bytes.len() < 8 || png_bytes[0..8] != png_header {
        return Err(DvmaError::ContainerError("Invalid PNG signature".into()));
    }

    let mut cursor = 8;

    while cursor < png_bytes.len() {
        if cursor + 8 > png_bytes.len() {
            return Err(DvmaError::ContainerError(
                "Truncated PNG chunk header".into(),
            ));
        }

        let length = u32::from_be_bytes(png_bytes[cursor..cursor + 4].try_into().unwrap()) as usize;
        let name = &png_bytes[cursor + 4..cursor + 8];

        if name == chunk_name {
            if cursor + 12 + length > png_bytes.len() {
                return Err(DvmaError::ContainerError(
                    "Truncated custom chunk data".into(),
                ));
            }

            let data = &png_bytes[cursor + 8..cursor + 8 + length];
            let crc_bytes = &png_bytes[cursor + 8 + length..cursor + 12 + length];
            let expected_crc = u32::from_be_bytes(crc_bytes.try_into().unwrap());

            let mut hasher = crc32fast::Hasher::new();
            hasher.update(&chunk_name);
            hasher.update(data);
            if hasher.finalize() != expected_crc {
                return Err(DvmaError::ContainerError(
                    "Custom PNG chunk CRC32 mismatch".into(),
                ));
            }

            return Ok(data.to_vec());
        }

        let chunk_total = 12 + length;
        if cursor + chunk_total > png_bytes.len() {
            return Err(DvmaError::ContainerError("Truncated PNG chunk data".into()));
        }
        cursor += chunk_total;
    }

    Err(DvmaError::ContainerError(format!(
        "Custom chunk '{:?}' not found in PNG file",
        std::str::from_utf8(&chunk_name).unwrap_or("????")
    )))
}
