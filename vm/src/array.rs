use crate::{array_entry_type::ArrayEntryType, value::Value, vm_error::VmError};

/// A java array, allocated on our memory chunk
pub trait Array<'a> {
    fn elements_type(&self) -> ArrayEntryType;

    fn len(&self) -> u32;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Errors will be returned if the type of the given value does not match the array type, or if the index is invalid
    fn set_element(&self, index: usize, value: Value<'a>) -> Result<(), VmError>;

    /// Errors will be returned if the index is invalid
    fn get_element(&self, index: usize) -> Result<Value<'a>, VmError>;
}
