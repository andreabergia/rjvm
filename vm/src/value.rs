use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use rjvm_reader::field_type::{BaseType, FieldType};

use crate::class::ClassPtr;

// TODO: do we need short/char/byte? What about boolean?
#[derive(Debug, Default, Clone)]
pub enum Value {
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
    Object(ObjectRef),
    // TODO: return address?
    // TODO: array?
}

pub struct ObjectValue {
    pub class: ClassPtr,
    pub fields: Vec<Value>,
}

impl Value {
    pub fn matches_type(&self, expected_type: FieldType) -> bool {
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
                FieldType::Object(class_name) => object_ref.borrow().class.name == class_name,
                _ => false,
            },
        }
    }
}

impl Debug for ObjectValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "class: {} fields {:?}", self.class.name, self.fields)
    }
}

// TODO: do we need the RefCell? Can we live with only the Rc?
pub type ObjectRef = Rc<RefCell<ObjectValue>>;
