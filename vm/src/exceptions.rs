use crate::{abstract_object::AbstractObject, value_stack::ValueStackError, vm_error::VmError};

/// Models the fact that a method execution has failed
#[derive(Debug, PartialEq)]
pub enum MethodCallFailed<'a> {
    InternalError(VmError),
    ExceptionThrown(JavaException<'a>),
}

impl<'a> From<VmError> for MethodCallFailed<'a> {
    fn from(value: VmError) -> Self {
        Self::InternalError(value)
    }
}

// TODO: need to remove this eventually and manage it with real exceptions
impl<'a> From<ValueStackError> for MethodCallFailed<'a> {
    fn from(_: ValueStackError) -> Self {
        Self::InternalError(VmError::ValidationException)
    }
}

/// Newtype that wraps a java exception
#[derive(Debug, PartialEq)]
pub struct JavaException<'a>(pub AbstractObject<'a>);
