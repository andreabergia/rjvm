


use crate::abstract_object::AbstractObject;
use crate::class::{ClassId, ClassRef};
use crate::value::Value;

pub trait Object<'a> {
    fn class_id(&self) -> ClassId;

    fn set_field(&self, index: usize, value: Value<'a>);

    fn get_field(&self, object_class: ClassRef, index: usize) -> Value<'a>;

    fn as_abstract_object(&self) -> AbstractObject;
}
