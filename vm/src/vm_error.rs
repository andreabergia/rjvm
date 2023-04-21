use thiserror::Error;

use rjvm_reader::class_reader_error::ClassReaderError;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum VmError {
    #[error("unexpected error loading class")]
    ClassLoadingError,

    #[error("null pointer exception")]
    NullPointerException,

    #[error("class not found: {0}")]
    ClassNotFoundException(String),

    #[error("method not found: {0}.{1}#{2}")]
    MethodNotFoundException(String, String, String),

    #[error("field not found: {0}.{1}")]
    FieldNotFoundException(String, String),

    #[error("validation exception - invalid class file")]
    ValidationException,

    #[error("arithmetic exception")]
    ArithmeticException,

    #[error("not yet implemented")]
    NotImplemented,

    #[error("array index out of bounds")]
    ArrayIndexOutOfBoundsException,

    #[error("class cast exception")]
    ClassCastException,

    #[error("class reading error: {0}")]
    ClassReaderError(ClassReaderError),
}

impl From<ClassReaderError> for VmError {
    fn from(value: ClassReaderError) -> Self {
        Self::ClassReaderError(value)
    }
}
