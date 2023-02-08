use crate::reader::class_file::ClassFile;
use std::cell::RefCell;
use std::rc::Rc;

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

#[derive(Debug)]
pub struct ObjectValue {
    pub class: Rc<ClassFile>,
    pub fields: Vec<Value>,
}

// TODO: do we need the RefCell? Can we live with only the Rc?
pub type ObjectRef = Rc<RefCell<ObjectValue>>;
