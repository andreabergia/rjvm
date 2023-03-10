use std::cell::RefCell;
use std::fmt::{Debug, Formatter};

use rjvm_reader::field_type::{BaseType, FieldType};

use crate::class::{Class, ClassId};
use crate::class_allocator::ClassResolver;

// TODO: do we need short/char/byte? What about boolean?
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Value<'a> {
    #[default]
    Uninitialized,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Char(i16),
    Float(i32),
    Double(i64),
    Boolean(bool),
    Object(ObjectRef<'a>),
    Null,
    // TODO: return address?
    // TODO: array?
}

#[derive(Clone, PartialEq, Eq)]
pub struct ObjectValue<'a> {
    pub class_id: ClassId,
    fields: RefCell<Vec<Value<'a>>>,
}

impl<'a> ObjectValue<'a> {
    pub fn new(class: &Class<'a>) -> Self {
        let fields = class.fields.iter().map(|_| Value::Uninitialized).collect();
        Self {
            class_id: class.id,
            fields: RefCell::new(fields),
        }
    }

    pub fn set_field(&self, index: usize, value: Value<'a>) {
        self.fields.borrow_mut()[index] = value;
    }

    pub fn get_field(&self, index: usize) -> Value<'a> {
        self.fields.borrow()[index].clone()
    }
}

pub type ObjectRef<'a> = &'a ObjectValue<'a>;

impl<'a> Value<'a> {
    pub fn matches_type<'b>(
        &self,
        expected_type: FieldType,
        class_resolver: &impl ClassResolver<'b>,
    ) -> bool {
        match self {
            Value::Uninitialized => false,
            Value::Byte(_) => match expected_type {
                FieldType::Base(base_type) => base_type == BaseType::Byte,
                _ => false,
            },
            Value::Short(_) => match expected_type {
                FieldType::Base(base_type) => base_type == BaseType::Short,
                _ => false,
            },
            Value::Int(_) => match expected_type {
                FieldType::Base(base_type) => base_type == BaseType::Int,
                _ => false,
            },
            Value::Long(_) => match expected_type {
                FieldType::Base(base_type) => base_type == BaseType::Long,
                _ => false,
            },
            Value::Char(_) => match expected_type {
                FieldType::Base(base_type) => base_type == BaseType::Char,
                _ => false,
            },
            Value::Float(_) => match expected_type {
                FieldType::Base(base_type) => base_type == BaseType::Float,
                _ => false,
            },
            Value::Double(_) => match expected_type {
                FieldType::Base(base_type) => base_type == BaseType::Double,
                _ => false,
            },
            Value::Boolean(_) => match expected_type {
                FieldType::Base(base_type) => base_type == BaseType::Boolean,
                _ => false,
            },

            Value::Object(object_ref) => match expected_type {
                // TODO: with multiple class loaders, we should check the class identity,
                //  not the name, since the same class could be loaded by multiple class loader
                FieldType::Object(class_name) => {
                    let value_class = class_resolver.find_class_by_id(object_ref.class_id);
                    if let Some(class_ref) = value_class {
                        class_ref.name == class_name
                    } else {
                        false
                    }
                }
                _ => false,
            },
            Value::Null => false,
        }
    }
}

impl<'a> Debug for ObjectValue<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "class: {} fields {:?}", self.class_id, self.fields)
    }
}
