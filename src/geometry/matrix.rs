use image::{GrayImage, Luma};
use std::f64::consts::PI;

use crate::core::error::{DvmaError, Result};
use crate::geometry::alphabet::{angle_to_letter, letter_to_angle};

pub const CELL_SIZE: u32 = 24;
pub const QUIET_ZONE_CELLS: u32 = 1;
pub const HEADER_LEN_CELLS: usize = 4; // 4 Base26 chars for length -> max 456,975 chars

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellType {
    AnchorTL,
    AnchorTR,
    AnchorBL,
    AnchorBR,
    HeaderLen(usize), // index 0..3
    Data(usize),
    Padding,
}

pub struct MatrixConfig {
    pub grid_dim: u32,
    pub cell_size: u32,
    pub dot_radius: f64,
    pub notch_depth: f64,
    pub notch_half_angle: f64,
}

impl Default for MatrixConfig {
    fn default() -> Self {
        Self {
            grid_dim: 16,
            cell_size: CELL_SIZE,
            dot_radius: 9.5,
            notch_depth: 4.0,
            notch_half_angle: 12.0 * PI / 180.0, // 24° total notch wedge
        }
    }
}

pub fn calculate_grid_dimension(payload_char_count: usize) -> Result<u32> {
    let total_required = 4 + HEADER_LEN_CELLS + payload_char_count;
    let mut dim = 4u32;
    while ((dim * dim) as usize) < total_required {
        dim += 1;
    }

    if dim > 128 {
        return Err(DvmaError::GeometryError(format!(
            "Payload too large: requires grid dimension {}x{} which exceeds maximum 128x128",
            dim, dim
        )));
    }

    Ok(dim)
}

/// Encodes an integer value up to 456,975 into exactly 4 Base26 uppercase A-Z characters.
pub fn encode_header_len(len: usize) -> Result<[char; 4]> {
    if len > 25 * 17576 + 25 * 676 + 25 * 26 + 25 {
        return Err(DvmaError::GeometryError(format!(
            "Payload length {} exceeds maximum header encoding capacity (456,975 chars)",
            len
        )));
    }

    let c0 = (len / 17576) % 26;
    let c1 = (len / 676) % 26;
    let c2 = (len / 26) % 26;
    let c3 = len % 26;

    Ok([
        (b'A' + c0 as u8) as char,
        (b'A' + c1 as u8) as char,
        (b'A' + c2 as u8) as char,
        (b'A' + c3 as u8) as char,
    ])
}

/// Decodes 4 Base26 uppercase A-Z characters back into integer length.
pub fn decode_header_len(chars: [char; 4]) -> usize {
    let v0 = (chars[0] as u8 - b'A') as usize;
    let v1 = (chars[1] as u8 - b'A') as usize;
    let v2 = (chars[2] as u8 - b'A') as usize;
    let v3 = (chars[3] as u8 - b'A') as usize;

    v0 * 17576 + v1 * 676 + v2 * 26 + v3
}

pub fn render_matrix(encoded_payload: &str, config: &MatrixConfig) -> Result<GrayImage> {
    let payload_chars: Vec<char> = encoded_payload.chars().collect();
    let dim = calculate_grid_dimension(payload_chars.len())?;

    let total_grid = dim + 2 * QUIET_ZONE_CELLS;
    let img_dim = total_grid * config.cell_size;

    let mut img = GrayImage::from_pixel(img_dim, img_dim, Luma([255u8]));

    let payload_len = payload_chars.len();
    let header_len_chars = encode_header_len(payload_len)?;

    let mut payload_idx = 0;

    for row in 0..dim {
        for col in 0..dim {
            let cell_type = match (row, col) {
                (0, 0) => CellType::AnchorTL,
                (0, c) if c == dim - 1 => CellType::AnchorTR,
                (r, 0) if r == dim - 1 => CellType::AnchorBL,
                (r, c) if r == dim - 1 && c == dim - 1 => CellType::AnchorBR,
                (0, 1) => CellType::HeaderLen(0),
                (0, 2) => CellType::HeaderLen(1),
                (0, 3) => CellType::HeaderLen(2),
                (0, 4) => CellType::HeaderLen(3),
                _ => {
                    if payload_idx < payload_chars.len() {
                        let idx = payload_idx;
                        payload_idx += 1;
                        CellType::Data(idx)
                    } else {
                        CellType::Padding
                    }
                }
            };

            let cell_x = (col + QUIET_ZONE_CELLS) * config.cell_size;
            let cell_y = (row + QUIET_ZONE_CELLS) * config.cell_size;

            render_cell(
                &mut img,
                cell_x,
                cell_y,
                cell_type,
                &payload_chars,
                header_len_chars,
                config,
            )?;
        }
    }

    Ok(img)
}

fn render_cell(
    img: &mut GrayImage,
    cell_x: u32,
    cell_y: u32,
    cell_type: CellType,
    payload_chars: &[char],
    header_len_chars: [char; 4],
    config: &MatrixConfig,
) -> Result<()> {
    let cx = cell_x as f64 + (config.cell_size as f64) / 2.0;
    let cy = cell_y as f64 + (config.cell_size as f64) / 2.0;

    match cell_type {
        CellType::AnchorTL => render_anchor(img, cell_x, cell_y, config.cell_size, 0),
        CellType::AnchorTR => render_anchor(img, cell_x, cell_y, config.cell_size, 1),
        CellType::AnchorBL => render_anchor(img, cell_x, cell_y, config.cell_size, 2),
        CellType::AnchorBR => render_anchor(img, cell_x, cell_y, config.cell_size, 3),
        CellType::HeaderLen(idx) => {
            let ch = header_len_chars[idx];
            render_dot_with_notch(img, cx, cy, ch, config)?;
            Ok(())
        }
        CellType::Data(idx) => {
            let ch = payload_chars[idx];
            render_dot_with_notch(img, cx, cy, ch, config)?;
            Ok(())
        }
        CellType::Padding => {
            render_dot_with_notch(img, cx, cy, 'A', config)?;
            Ok(())
        }
    }
}

fn render_anchor(
    img: &mut GrayImage,
    cell_x: u32,
    cell_y: u32,
    cell_size: u32,
    variant: u8,
) -> Result<()> {
    let cx = cell_x as f64 + (cell_size as f64) / 2.0;
    let cy = cell_y as f64 + (cell_size as f64) / 2.0;
    let r = (cell_size as f64) * 0.4;

    for y in cell_y..cell_y + cell_size {
        for x in cell_x..cell_x + cell_size {
            let dx = x as f64 + 0.5 - cx;
            let dy = y as f64 + 0.5 - cy;
            let dist = (dx * dx + dy * dy).sqrt();

            let black = match variant {
                0 => dx.abs() <= r && dy.abs() <= r,
                1 => {
                    (dx.abs() <= r && dy.abs() <= r && (dx.abs() >= r * 0.5 || dy.abs() >= r * 0.5))
                        || dist <= r * 0.25
                }
                2 => dist <= r,
                3 => (dist <= r && dist >= r * 0.6) || dist <= r * 0.25,
                _ => false,
            };

            if black {
                img.put_pixel(x, y, Luma([0u8]));
            }
        }
    }
    Ok(())
}

fn render_dot_with_notch(
    img: &mut GrayImage,
    cx: f64,
    cy: f64,
    ch: char,
    config: &MatrixConfig,
) -> Result<()> {
    let target_deg = letter_to_angle(ch)?;
    let target_rad = target_deg * PI / 180.0;

    let r_outer = config.dot_radius;
    let r_inner = config.dot_radius - config.notch_depth;

    let min_x = (cx - r_outer - 1.0).floor() as u32;
    let max_x = (cx + r_outer + 1.0).ceil() as u32;
    let min_y = (cy - r_outer - 1.0).floor() as u32;
    let max_y = (cy + r_outer + 1.0).ceil() as u32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f64 + 0.5 - cx;
            let dy = y as f64 + 0.5 - cy;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist <= r_outer {
                let mut phi = dx.atan2(-dy);
                if phi < 0.0 {
                    phi += 2.0 * PI;
                }

                let mut diff = (phi - target_rad).abs();
                if diff > PI {
                    diff = 2.0 * PI - diff;
                }

                let in_notch_wedge = diff <= config.notch_half_angle;

                if in_notch_wedge && dist > r_inner {
                    continue;
                } else {
                    img.put_pixel(x, y, Luma([0u8]));
                }
            }
        }
    }

    Ok(())
}

pub fn decode_matrix(img: &GrayImage, config: &MatrixConfig) -> Result<String> {
    let width = img.width();
    let height = img.height();

    if width != height {
        return Err(DvmaError::GeometryError(
            "Matrix image must be square".into(),
        ));
    }

    let total_grid = width / config.cell_size;
    if total_grid < 1 + 2 * QUIET_ZONE_CELLS {
        return Err(DvmaError::GeometryError(
            "Image dimensions too small".into(),
        ));
    }

    let dim = total_grid - 2 * QUIET_ZONE_CELLS;

    let h0 = decode_cell_notch(img, 1, 0, config)?;
    let h1 = decode_cell_notch(img, 2, 0, config)?;
    let h2 = decode_cell_notch(img, 3, 0, config)?;
    let h3 = decode_cell_notch(img, 4, 0, config)?;

    let payload_len = decode_header_len([h0, h1, h2, h3]);

    let mut recovered_chars = String::with_capacity(payload_len);
    let mut payload_idx = 0;

    for row in 0..dim {
        for col in 0..dim {
            let is_corner = (row == 0 || row == dim - 1) && (col == 0 || col == dim - 1);
            let is_header = row == 0 && (1..=4).contains(&col);

            if is_corner || is_header {
                continue;
            }

            if payload_idx < payload_len {
                let ch = decode_cell_notch(img, col, row, config)?;
                recovered_chars.push(ch);
                payload_idx += 1;
            }
        }
    }

    if recovered_chars.len() != payload_len {
        return Err(DvmaError::GeometryError(format!(
            "Failed to decode complete payload: expected {} chars, got {}",
            payload_len,
            recovered_chars.len()
        )));
    }

    Ok(recovered_chars)
}

fn decode_cell_notch(img: &GrayImage, col: u32, row: u32, config: &MatrixConfig) -> Result<char> {
    let cell_x = (col + QUIET_ZONE_CELLS) * config.cell_size;
    let cell_y = (row + QUIET_ZONE_CELLS) * config.cell_size;

    let cx = cell_x as f64 + (config.cell_size as f64) / 2.0;
    let cy = cell_y as f64 + (config.cell_size as f64) / 2.0;

    let r_inner = config.dot_radius - config.notch_depth;
    let r_outer = config.dot_radius;

    let mut sin_sum = 0.0;
    let mut cos_sum = 0.0;
    let mut white_pixel_count = 0;

    let min_x = (cx - r_outer - 1.0).floor() as i32;
    let max_x = (cx + r_outer + 1.0).ceil() as i32;
    let min_y = (cy - r_outer - 1.0).floor() as i32;
    let max_y = (cy + r_outer + 1.0).ceil() as i32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if x >= 0 && x < img.width() as i32 && y >= 0 && y < img.height() as i32 {
                let dx = x as f64 + 0.5 - cx;
                let dy = y as f64 + 0.5 - cy;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist >= r_inner - 0.25 && dist <= r_outer + 0.25 {
                    let val = img.get_pixel(x as u32, y as u32)[0];
                    if val > 128 {
                        let mut phi = dx.atan2(-dy);
                        if phi < 0.0 {
                            phi += 2.0 * PI;
                        }
                        sin_sum += phi.sin();
                        cos_sum += phi.cos();
                        white_pixel_count += 1;
                    }
                }
            }
        }
    }

    if white_pixel_count == 0 {
        return Err(DvmaError::GeometryError(format!(
            "No notch cutout pixels detected in cell at grid ({}, {})",
            col, row
        )));
    }

    let mut mean_rad = sin_sum.atan2(cos_sum);
    if mean_rad < 0.0 {
        mean_rad += 2.0 * PI;
    }
    let mean_deg = mean_rad * 180.0 / PI;

    angle_to_letter(mean_deg, 5.0)
}
