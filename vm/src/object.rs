use crate::{
    class::{ClassId, ClassRef},
    value::Value,
};

/// A java array, allocated on our memory chunk
pub trait Object<'a> {
    fn class_id(&self) -> ClassId;

    /// Errors will be returned if the type of the given value does not match the field type, or if the index is invalid
    fn set_field(&self, index: usize, value: Value<'a>);

    /// Errors will be returned if the index is invalid
    fn get_field(&self, object_class: ClassRef, index: usize) -> Value<'a>;
}
