use crate::core::error::{DvmaError, Result};

pub const MAGIC_BYTES: &[u8; 4] = b"DVMA";
pub const FORMAT_VERSION_1: u8 = 1;

/// Payload framing layer holding AEAD encrypted payload metadata and ciphertext.
/// Designed with versioning and reserved fields to support future extensions like Reed-Solomon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryPayload {
    pub version: u8,
    pub algorithm_id: u8, // 1 = ChaCha20-Poly1305
    pub nonce: [u8; 12],
    pub ciphertext_and_tag: Vec<u8>,
}

impl BinaryPayload {
    pub const ALGORITHM_CHACHA20_POLY1305: u8 = 1;

    pub fn new(nonce: [u8; 12], ciphertext_and_tag: Vec<u8>) -> Self {
        Self {
            version: FORMAT_VERSION_1,
            algorithm_id: Self::ALGORITHM_CHACHA20_POLY1305,
            nonce,
            ciphertext_and_tag,
        }
    }

    /// Serialize payload to binary bytes.
    /// Format: [MAGIC (4b)] [VERSION (1b)] [ALG_ID (1b)] [NONCE (12b)] [CIPHERTEXT_LEN (4b BE)] [CIPHERTEXT_AND_TAG]
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(4 + 1 + 1 + 12 + 4 + self.ciphertext_and_tag.len());
        bytes.extend_from_slice(MAGIC_BYTES);
        bytes.push(self.version);
        bytes.push(self.algorithm_id);
        bytes.extend_from_slice(&self.nonce);
        let len = self.ciphertext_and_tag.len() as u32;
        bytes.extend_from_slice(&len.to_be_bytes());
        bytes.extend_from_slice(&self.ciphertext_and_tag);
        bytes
    }

    /// Deserialize binary payload from bytes.
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        if data.len() < 4 + 1 + 1 + 12 + 4 {
            return Err(DvmaError::FramingError("Payload data too short".into()));
        }

        if &data[0..4] != MAGIC_BYTES {
            return Err(DvmaError::FramingError("Invalid magic identifier".into()));
        }

        let version = data[4];
        if version != FORMAT_VERSION_1 {
            return Err(DvmaError::FramingError(format!(
                "Unsupported format version {}",
                version
            )));
        }

        let algorithm_id = data[5];
        if algorithm_id != Self::ALGORITHM_CHACHA20_POLY1305 {
            return Err(DvmaError::FramingError(format!(
                "Unsupported algorithm ID {}",
                algorithm_id
            )));
        }

        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&data[6..18]);

        let len_bytes = &data[18..22];
        let ciphertext_len = u32::from_be_bytes(len_bytes.try_into().unwrap()) as usize;

        if data.len() < 22 + ciphertext_len {
            return Err(DvmaError::FramingError(
                "Truncated ciphertext payload".into(),
            ));
        }

        let ciphertext_and_tag = data[22..22 + ciphertext_len].to_vec();

        Ok(Self {
            version,
            algorithm_id,
            nonce,
            ciphertext_and_tag,
        })
    }
}
