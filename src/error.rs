use thiserror::Error;

#[derive(Error, Debug)]
pub enum MageError {
    #[error("Error: {0}")]
    Unexpected(String),

    #[error("File error: {0}")]
    File(String),

    #[error("Magefile not found")]
    MageFileNotFound,

    #[error("Invalid magefile: {0}")]
    InvalidMageFile(String),

    #[error("Invalid path or url {0}")]
    InvalidDotfilesOrigin(String),

    #[error("Invalid path {0}")]
    InvalidPath(String),
}

impl From<std::io::Error> for MageError {
    fn from(value: std::io::Error) -> Self {
        MageError::File(format!("{value}"))
    }
}
