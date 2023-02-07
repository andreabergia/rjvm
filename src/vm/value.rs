use crate::reader::class_file::ClassFile;

#[derive(Debug)]
pub enum Value {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Char(i16),
    Float(i32),
    Double(i64),
    Boolean(bool),
    Object(ObjectValue),
    // TODO: return address?
    // TODO: array?
}

#[derive(Debug)]
pub struct ObjectValue {
    class: Box<ClassFile>,
    fields: Vec<Value>,
}
