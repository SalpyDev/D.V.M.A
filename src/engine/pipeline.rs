use image::GrayImage;
use std::path::Path;
use zeroize::Zeroizing;

use crate::codec::Base26Codec;
use crate::container::{load_dvma_png, save_dvma_png};
use crate::core::error::{DvmaError, Result};
use crate::core::framing::BinaryPayload;
use crate::crypto::{
    decrypt_document, encrypt_document, generate_master_key, reconstruct_master_key,
    split_master_key, KeyShare,
};
use crate::geometry::{decode_matrix, render_matrix, MatrixConfig};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryState {
    Init,
    ImageAccepted,
    AwaitingShareA,
    ShareValidation,
    KeyReconstruction,
    PayloadDecoding,
    AuthenticatedDecryption,
    DocumentRestoration,
    RecoveryFailed,
    RecoverySucceeded,
}

pub struct DvmaCreationResult {
    pub share_a: KeyShare,
    pub share_a_user_string: String,
}

/// Creation Pipeline:
/// 1. Read document bytes.
/// 2. Generate master key K.
/// 3. Encrypt document with ChaCha20-Poly1305 -> BinaryPayload.
/// 4. Encode payload bytes as A-Z using Base26.
/// 5. Render matrix image from Base26 string.
/// 6. Split K into Share A and Share B.
/// 7. Package Share B into PNG custom chunk `dvMa` and write to output path.
/// 8. Reopen and validate PNG artifact.
pub fn create_artifact<P: AsRef<Path>>(
    input_doc_bytes: &[u8],
    output_png_path: P,
) -> Result<DvmaCreationResult> {
    if input_doc_bytes.is_empty() {
        return Err(DvmaError::FramingError("Input document is empty".into()));
    }

    // Generate K
    let master_key = generate_master_key();

    // AEAD Encrypt
    let payload = encrypt_document(&master_key, input_doc_bytes)?;
    let serialized_payload = payload.serialize();

    // Base26 Encode
    let base26_str = Base26Codec::encode(&serialized_payload);

    // Render Matrix
    let config = MatrixConfig::default();
    let matrix_img = render_matrix(&base26_str, &config)?;

    // Split Key
    let (share_a, share_b) = split_master_key(&master_key);
    let share_a_user_str = share_a.to_user_string();

    // Save PNG artifact
    save_dvma_png(&output_png_path, &matrix_img, &share_b)?;

    // Reopen & Validate Artifact
    let (loaded_img, loaded_share_b) = load_dvma_png(&output_png_path)?;
    if loaded_share_b.share_set_id != share_b.share_set_id {
        return Err(DvmaError::ContainerError(
            "Artifact validation failed: Share B share_set_id mismatch".into(),
        ));
    }

    let decoded_base26 = decode_matrix(&loaded_img, &config)?;
    if decoded_base26 != base26_str {
        return Err(DvmaError::ContainerError(
            "Artifact validation failed: decoded Base26 matrix string mismatch".into(),
        ));
    }

    Ok(DvmaCreationResult {
        share_a,
        share_a_user_string: share_a_user_str,
    })
}

/// Recovery State Machine with mandatory Share A reconciliation gate.
pub struct RecoveryEngine {
    pub state: RecoveryState,
    loaded_img: Option<GrayImage>,
    share_b: Option<KeyShare>,
    share_a: Option<KeyShare>,
    candidate_key: Option<Zeroizing<[u8; 32]>>,
}

impl Default for RecoveryEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RecoveryEngine {
    pub fn new() -> Self {
        Self {
            state: RecoveryState::Init,
            loaded_img: None,
            share_b: None,
            share_a: None,
            candidate_key: None,
        }
    }

    /// Step 1: Ingest PNG image and extract Share B.
    pub fn ingest_image<P: AsRef<Path>>(&mut self, png_path: P) -> Result<()> {
        let (img, share_b) = load_dvma_png(png_path)?;
        self.loaded_img = Some(img);
        self.share_b = Some(share_b);
        self.state = RecoveryState::ImageAccepted;

        // Transition to AwaitingShareA gate
        self.state = RecoveryState::AwaitingShareA;
        Ok(())
    }

    /// Step 2: Supply Share A (Reconciliation Gate).
    pub fn supply_share_a(&mut self, share_a_input: &str) -> Result<()> {
        if self.state != RecoveryState::AwaitingShareA {
            return Err(DvmaError::StateError(
                "Cannot supply Share A: engine is not in AwaitingShareA state".into(),
            ));
        }

        self.state = RecoveryState::ShareValidation;
        let share_a = KeyShare::from_user_string(share_a_input)?;

        let share_b = self
            .share_b
            .as_ref()
            .ok_or_else(|| DvmaError::StateError("Share B missing from ingested image".into()))?;

        if share_a.share_set_id != share_b.share_set_id {
            self.state = RecoveryState::RecoveryFailed;
            return Err(DvmaError::ShamirError(
                "Share A and Share B belong to different key sets".into(),
            ));
        }

        self.share_a = Some(share_a);
        self.state = RecoveryState::KeyReconstruction;

        // Reconstruct candidate K
        let candidate_key = reconstruct_master_key(self.share_a.as_ref().unwrap(), share_b)?;
        self.candidate_key = Some(candidate_key);

        Ok(())
    }

    /// Step 3: Decode matrix, decrypt payload, and restore original document bytes.
    /// Fails closed with DvmaError::AuthenticationFailed if key or payload is invalid.
    pub fn execute_recovery(&mut self) -> Result<Zeroizing<Vec<u8>>> {
        if self.candidate_key.is_none() || self.share_a.is_none() {
            self.state = RecoveryState::RecoveryFailed;
            return Err(DvmaError::StateError(
                "Mandatory Share A gate not satisfied: candidate key not reconstructed".into(),
            ));
        }

        let img = self
            .loaded_img
            .as_ref()
            .ok_or_else(|| DvmaError::StateError("No image loaded for decoding".into()))?;

        self.state = RecoveryState::PayloadDecoding;
        let config = MatrixConfig::default();
        let base26_str = decode_matrix(img, &config)?;

        let payload_bytes = Base26Codec::decode(&base26_str)?;
        let binary_payload = BinaryPayload::deserialize(&payload_bytes)?;

        self.state = RecoveryState::AuthenticatedDecryption;
        let candidate_key = self.candidate_key.as_ref().unwrap();

        let restored_doc = match decrypt_document(candidate_key, &binary_payload) {
            Ok(doc) => doc,
            Err(e) => {
                self.state = RecoveryState::RecoveryFailed;
                return Err(e);
            }
        };

        self.state = RecoveryState::DocumentRestoration;
        self.state = RecoveryState::RecoverySucceeded;

        Ok(restored_doc)
    }
}
