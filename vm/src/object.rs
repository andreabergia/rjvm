use crate::{
    class::{ClassId, ClassRef},
    value::Value,
};

pub trait Object<'a> {
    fn class_id(&self) -> ClassId;

    fn set_field(&self, index: usize, value: Value<'a>);

    fn get_field(&self, object_class: ClassRef, index: usize) -> Value<'a>;
}
