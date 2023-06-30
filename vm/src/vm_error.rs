use thiserror::Error;

use crate::value_stack::ValueStackError;

/// Various errors that are thrown when executing java bytecode
// TODO: this implementation is quite poor: we do not keep track of the origin
//  of the errors, and we do not keep many details
#[derive(Debug, Error, PartialEq, Eq)]
pub enum VmError {
    #[error("unexpected error loading class: {0}")]
    ClassLoadingError(String),

    /// TODO: this should become throwing a real `java.lang.NullPointerException`
    #[error("null pointer exception")]
    NullPointerException,

    /// TODO: this should become throwing a real `java.lang.ClassNotFoundException`
    #[error("class not found: {0}")]
    ClassNotFoundException(String),

    #[error("method not found: {0}.{1}#{2}")]
    MethodNotFoundException(String, String, String),

    #[error("field not found: {0}.{1}")]
    FieldNotFoundException(String, String),

    /// This is an overly generic error, abused to mean "something unexpected happened".
    /// It includes mostly errors that should be checked during the linking phase of the class file
    /// (which we have not implemented).
    #[error("validation exception - invalid class file")]
    ValidationException,

    /// TODO: this should become throwing a real `java.lang.ArithmeticException`
    #[error("arithmetic exception")]
    ArithmeticException,

    #[error("not yet implemented")]
    NotImplemented,

    /// TODO: this should become throwing a real `java.lang.ArrayIndexOutOfBoundsException`
    #[error("array index out of bounds")]
    ArrayIndexOutOfBoundsException,

    /// TODO: this should become throwing a real `java.lang.ClassCastException`
    #[error("class cast exception")]
    ClassCastException,
}

// TODO: remove once we implement exceptions
impl From<ValueStackError> for VmError {
    fn from(_: ValueStackError) -> Self {
        Self::ValidationException
    }
}
