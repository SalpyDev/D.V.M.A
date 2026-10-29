use thiserror::Error;

#[derive(Error, Debug)]
pub enum DvmaError {
    #[error("Base26 encoding error: {0}")]
    Base26EncodingError(String),

    #[error("Base26 decoding error: {0}")]
    Base26DecodingError(String),

    #[error("Cryptographic error: {0}")]
    CryptoError(String),

    #[error("Shamir secret sharing error: {0}")]
    ShamirError(String),

    #[error("Geometry / Matrix error: {0}")]
    GeometryError(String),

    #[error("PNG container error: {0}")]
    ContainerError(String),

    #[error("Framing / Version error: {0}")]
    FramingError(String),

    #[error("State machine error: {0}")]
    StateError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid Share A: {0}")]
    InvalidShareA(String),

    #[error("Authentication failed: invalid key, tampered payload, or wrong share")]
    AuthenticationFailed,
}

pub type Result<T> = std::result::Result<T, DvmaError>;
