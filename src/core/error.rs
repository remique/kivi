use thiserror::Error;

#[derive(Error, Debug)]
pub enum KiviError {
    #[error("Error: {0}")]
    Generic(String),

    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serde_json error: {0}")]
    Serde(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, KiviError>;
