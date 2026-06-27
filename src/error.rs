use thiserror::Error;

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
