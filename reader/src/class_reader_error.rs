use thiserror::Error;

use crate::{constant_pool::InvalidConstantPoolIndexError, instruction::Instruction};
use rjvm_utils::buffer::BufferError;

#[derive(Error, Debug, PartialEq)]
pub enum ClassReaderError {
    #[error("generic I/O error: {0}")]
    IoError(String),

    #[error("invalid class file: {0}")]
    InvalidClassData(String),

    #[error("unsupported class file version {0}.{1}")]
    UnsupportedVersion(u16, u16),

    #[error("unsupported instruction: {0:?}")]
    UnsupportedInstruction(Instruction),

    #[error("validation error: {0}")]
    ValidationError(String),

    #[error("Invalid type descriptor: {0}")]
    InvalidTypeDescriptor(String),
}

pub type Result<T> = std::result::Result<T, ClassReaderError>;

impl From<InvalidConstantPoolIndexError> for ClassReaderError {
    fn from(value: InvalidConstantPoolIndexError) -> Self {
        Self::InvalidClassData(value.to_string())
    }
}

impl From<std::io::Error> for ClassReaderError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(format!("{err}"))
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
