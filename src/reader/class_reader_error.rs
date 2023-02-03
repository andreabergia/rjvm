use thiserror::Error;

use crate::reader::constant_pool::InvalidConstantPoolIndexError;

#[derive(Error, Debug, PartialEq)]
pub enum ClassReaderError {
    #[error("generic I/O error: {0}")]
    IoError(String),
    #[error("invalid class file: {0}")]
    InvalidClassData(String),

    #[error("unsupported class file version {0}.{1}")]
    UnsupportedVersion(u16, u16),
}

pub type Result<T> = std::result::Result<T, ClassReaderError>;

impl From<InvalidConstantPoolIndexError> for ClassReaderError {
    fn from(value: InvalidConstantPoolIndexError) -> Self {
        Self::InvalidClassData(value.to_string())
    }
}

impl From<std::io::Error> for ClassReaderError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(format!("{}", err))
    }
}
