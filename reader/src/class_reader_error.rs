use thiserror::Error;

use crate::constant_pool::InvalidConstantPoolIndexError;
use rjvm_utils::buffer::BufferError;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ClassReaderError {
    #[error("invalid class file: {0}")]
    InvalidClassData(String),

    #[error("unsupported class file version {0}.{1}")]
    UnsupportedVersion(u16, u16),

    #[error("Invalid type descriptor: {0}")]
    InvalidTypeDescriptor(String),
}

pub type Result<T> = std::result::Result<T, ClassReaderError>;

impl From<InvalidConstantPoolIndexError> for ClassReaderError {
    fn from(value: InvalidConstantPoolIndexError) -> Self {
        Self::InvalidClassData(value.to_string())
    }
}

impl From<BufferError> for ClassReaderError {
    fn from(err: BufferError) -> Self {
        match err {
            BufferError::UnexpectedEndOfClassFile => {
                Self::InvalidClassData("unexpected end of class file".to_string())
            }
            BufferError::InvalidCesu8String => {
                Self::InvalidClassData("invalid cesu8 string".to_string())
            }
        }
    }
}
