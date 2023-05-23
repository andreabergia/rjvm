use crate::value::ObjectValue;
use crate::{value_stack::ValueStackError, vm_error::VmError};

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

#[derive(Debug, PartialEq)]
pub struct JavaException<'a>(pub ObjectValue<'a>);
