use thiserror::Error;

/// Crate-level Result type alias for convenience.
pub type Result<T> = std::result::Result<T, TilerError>;

/// Library-specific error representations.
#[derive(Error, Debug)]
pub enum TilerError {
    #[error("I/O error occurred: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image manipulation error: {0}")]
    Image(#[from] image::ImageError),

    #[error("JSON processing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid configuration value: {0}")]
    InvalidConfig(String),
}