use crate::reader::class_file::ClassFile;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

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
    pub class: Rc<ClassFile>,
    pub fields: Vec<Value>,
}

impl Debug for ObjectValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "class: {} fields {:?}", self.class.name, self.fields)
    }
}

// TODO: do we need the RefCell? Can we live with only the Rc?
pub type ObjectRef = Rc<RefCell<ObjectValue>>;
