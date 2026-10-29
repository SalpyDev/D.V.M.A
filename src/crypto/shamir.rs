use crc32fast::Hasher;
use rand::rngs::OsRng;
use rand::RngCore;
use zeroize::{Zeroize, Zeroizing};

use crate::core::error::{DvmaError, Result};
use crate::crypto::aead::MASTER_KEY_LEN;
use crate::crypto::gf256::{gf256_add, gf256_div, gf256_mul};

pub const SHARE_SET_ID_LEN: usize = 8;
pub const SHARE_A_X: u8 = 1;
pub const SHARE_B_X: u8 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Zeroize)]
#[zeroize(drop)]
pub struct KeyShare {
    pub share_set_id: [u8; SHARE_SET_ID_LEN],
    pub x_coord: u8,
    pub data: Vec<u8>,
}

impl KeyShare {
    pub fn new(share_set_id: [u8; SHARE_SET_ID_LEN], x_coord: u8, data: Vec<u8>) -> Self {
        Self {
            share_set_id,
            x_coord,
            data,
        }
    }

    /// Serializes key share into a structured byte buffer with CRC32 checksum.
    /// Format: [SHARE_SET_ID (8b)] [X_COORD (1b)] [DATA_LEN (2b BE)] [DATA (32b)] [CRC32 (4b BE)]
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(8 + 1 + 2 + self.data.len() + 4);
        buf.extend_from_slice(&self.share_set_id);
        buf.push(self.x_coord);
        let len = self.data.len() as u16;
        buf.extend_from_slice(&len.to_be_bytes());
        buf.extend_from_slice(&self.data);

        let mut hasher = Hasher::new();
        hasher.update(&buf);
        let crc = hasher.finalize();
        buf.extend_from_slice(&crc.to_be_bytes());

        buf
    }

    /// Deserializes key share from a byte buffer and verifies CRC32 checksum.
    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 8 + 1 + 2 + 4 {
            return Err(DvmaError::ShamirError("Share bytes too short".into()));
        }

        let payload_len = bytes.len() - 4;
        let payload = &bytes[..payload_len];
        let crc_bytes = &bytes[payload_len..];
        let expected_crc = u32::from_be_bytes(crc_bytes.try_into().unwrap());

        let mut hasher = Hasher::new();
        hasher.update(payload);
        if hasher.finalize() != expected_crc {
            return Err(DvmaError::ShamirError(
                "Share CRC32 checksum mismatch".into(),
            ));
        }

        let mut share_set_id = [0u8; SHARE_SET_ID_LEN];
        share_set_id.copy_from_slice(&payload[0..8]);
        let x_coord = payload[8];
        let data_len = u16::from_be_bytes(payload[9..11].try_into().unwrap()) as usize;

        if payload.len() < 11 + data_len {
            return Err(DvmaError::ShamirError("Share payload truncated".into()));
        }

        let data = payload[11..11 + data_len].to_vec();

        Ok(Self {
            share_set_id,
            x_coord,
            data,
        })
    }

    /// Formats Share A as a user-friendly string: `DVMA-SHARE-A-<HEX_SERIALIZED>`
    pub fn to_user_string(&self) -> String {
        format!("DVMA-SHARE-A-{}", hex::encode_upper(self.serialize()))
    }

    /// Parses Share A from user input string.
    pub fn from_user_string(input: &str) -> Result<Self> {
        let trimmed = input.trim();
        let hex_str = if let Some(stripped) = trimmed.strip_prefix("DVMA-SHARE-A-") {
            stripped
        } else {
            trimmed
        };

        let bytes = hex::decode(hex_str)
            .map_err(|e| DvmaError::InvalidShareA(format!("Invalid hex string: {}", e)))?;

        let share = Self::deserialize(&bytes)
            .map_err(|e| DvmaError::InvalidShareA(format!("Malformed share payload: {}", e)))?;

        if share.x_coord != SHARE_A_X {
            return Err(DvmaError::InvalidShareA(format!(
                "Expected Share A (x=1), but share coordinate is x={}",
                share.x_coord
            )));
        }

        Ok(share)
    }
}

/// Splits a 32-byte master key K into Share A (x=1) and Share B (x=2) using 2-of-2 Shamir over GF(2^8).
pub fn split_master_key(master_key: &[u8; MASTER_KEY_LEN]) -> (KeyShare, KeyShare) {
    let mut share_set_id = [0u8; SHARE_SET_ID_LEN];
    OsRng.fill_bytes(&mut share_set_id);

    let mut coeff_a = [0u8; MASTER_KEY_LEN];
    OsRng.fill_bytes(&mut coeff_a);

    let mut data_a = vec![0u8; MASTER_KEY_LEN];
    let mut data_b = vec![0u8; MASTER_KEY_LEN];

    for i in 0..MASTER_KEY_LEN {
        let k_i = master_key[i];
        let a_i = coeff_a[i];

        // f_i(x) = k_i + a_i * x
        // Share A: x = 1 -> k_i + a_i * 1 = k_i ^ a_i
        let y_a = gf256_add(k_i, gf256_mul(a_i, SHARE_A_X));
        // Share B: x = 2 -> k_i + a_i * 2
        let y_b = gf256_add(k_i, gf256_mul(a_i, SHARE_B_X));

        data_a[i] = y_a;
        data_b[i] = y_b;
    }

    let share_a = KeyShare::new(share_set_id, SHARE_A_X, data_a);
    let share_b = KeyShare::new(share_set_id, SHARE_B_X, data_b);

    (share_a, share_b)
}

/// Reconstructs the 32-byte master key K from Share A and Share B using Lagrange interpolation over GF(2^8).
pub fn reconstruct_master_key(
    share_a: &KeyShare,
    share_b: &KeyShare,
) -> Result<Zeroizing<[u8; MASTER_KEY_LEN]>> {
    if share_a.share_set_id != share_b.share_set_id {
        return Err(DvmaError::ShamirError(
            "Share set ID mismatch: Share A and Share B belong to different key sets".into(),
        ));
    }

    if share_a.data.len() != MASTER_KEY_LEN || share_b.data.len() != MASTER_KEY_LEN {
        return Err(DvmaError::ShamirError(format!(
            "Invalid share data length: expected {}, got A={}, B={}",
            MASTER_KEY_LEN,
            share_a.data.len(),
            share_b.data.len()
        )));
    }

    let x1 = share_a.x_coord;
    let x2 = share_b.x_coord;

    if x1 == x2 {
        return Err(DvmaError::ShamirError(
            "Duplicate share coordinates supplied".into(),
        ));
    }

    // Lagrange constants at x = 0:
    // c1 = x2 / (x1 + x2)
    // c2 = x1 / (x1 + x2)
    let denom = gf256_add(x1, x2);
    let c1 = gf256_div(x2, denom);
    let c2 = gf256_div(x1, denom);

    let mut master_key = Zeroizing::new([0u8; MASTER_KEY_LEN]);

    for i in 0..MASTER_KEY_LEN {
        let y1 = share_a.data[i];
        let y2 = share_b.data[i];

        // k_i = y1 * c1 + y2 * c2
        let term1 = gf256_mul(y1, c1);
        let term2 = gf256_mul(y2, c2);
        master_key[i] = gf256_add(term1, term2);
    }

    Ok(master_key)
}
