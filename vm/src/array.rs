use crate::{
    abstract_object::AbstractObject, array_entry_type::ArrayEntryType, value::Value,
    vm_error::VmError,
};

pub trait Array<'a> {
    fn elements_type(&self) -> ArrayEntryType;

    fn len(&self) -> u32;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn set_element(&self, index: usize, value: Value<'a>) -> Result<(), VmError>;

    fn get_element(&self, index: usize) -> Result<Value<'a>, VmError>;

    fn as_abstract_object(&self) -> AbstractObject;
}
