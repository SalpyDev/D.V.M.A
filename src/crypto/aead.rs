use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use rand::rngs::OsRng;
use rand::RngCore;
use zeroize::Zeroizing;

use crate::core::error::{DvmaError, Result};
use crate::core::framing::BinaryPayload;

pub const MASTER_KEY_LEN: usize = 32;
pub const NONCE_LEN: usize = 12;

/// Generates a fresh cryptographically secure 256-bit master key K.
pub fn generate_master_key() -> Zeroizing<[u8; MASTER_KEY_LEN]> {
    let mut key = Zeroizing::new([0u8; MASTER_KEY_LEN]);
    OsRng.fill_bytes(key.as_mut());
    key
}

/// Encrypts plaintext bytes using ChaCha20-Poly1305 and a fresh CSPRNG 96-bit nonce.
/// Returns a BinaryPayload struct.
pub fn encrypt_document(
    master_key: &[u8; MASTER_KEY_LEN],
    plaintext: &[u8],
) -> Result<BinaryPayload> {
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);

    let cipher = ChaCha20Poly1305::new_from_slice(master_key)
        .map_err(|e| DvmaError::CryptoError(e.to_string()))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext_and_tag = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| DvmaError::CryptoError(format!("Encryption failed: {}", e)))?;

    Ok(BinaryPayload::new(nonce_bytes, ciphertext_and_tag))
}

/// Decrypts a BinaryPayload using candidate master_key K.
/// Performs AEAD authentication tag check. Returns plaintext bytes on success.
/// Returns DvmaError::AuthenticationFailed if decryption / tag check fails.
pub fn decrypt_document(
    master_key: &[u8; MASTER_KEY_LEN],
    payload: &BinaryPayload,
) -> Result<Zeroizing<Vec<u8>>> {
    if payload.algorithm_id != BinaryPayload::ALGORITHM_CHACHA20_POLY1305 {
        return Err(DvmaError::CryptoError(format!(
            "Unsupported algorithm ID {}",
            payload.algorithm_id
        )));
    }

    let cipher = ChaCha20Poly1305::new_from_slice(master_key)
        .map_err(|e| DvmaError::CryptoError(e.to_string()))?;
    let nonce = Nonce::from_slice(&payload.nonce);

    let plaintext = cipher
        .decrypt(nonce, payload.ciphertext_and_tag.as_slice())
        .map_err(|_| DvmaError::AuthenticationFailed)?;

    Ok(Zeroizing::new(plaintext))
}
