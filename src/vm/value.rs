use crate::reader::class_file::ClassFile;

pub enum Value {
    Int(i32),
    Long(i64),
    Float(i32),
    Double(i64),
    Object(ObjectValue),
    // TODO: array?
}

pub struct ObjectValue {
    class: Box<ClassFile>,
    fields: Vec<Value>,
}
