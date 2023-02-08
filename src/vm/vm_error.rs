use thiserror::Error;

use crate::reader::class_reader_error::ClassReaderError;

#[derive(Debug, Error)]
pub enum VmError {
    #[error("null pointer exception")]
    NullPointerException,

    #[error("class not found: {0}")]
    ClassNotFoundException(String),

    #[error("validation exception - invalid class file")]
    ValidationException,

    #[error("not yet implemented")]
    NotImplemented,

    #[error("unexpected error")]
    UnexpectedError,
}

impl From<ClassReaderError> for VmError {
    fn from(value: ClassReaderError) -> Self {
        match value {
            ClassReaderError::ValidationError(_) => Self::ValidationException,
            _ => Self::UnexpectedError,
        }
    }
}
