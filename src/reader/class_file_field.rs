use std::fmt;
use std::fmt::Formatter;

use crate::reader::field_flags::FieldFlags;

#[derive(Debug, PartialEq)]
pub struct ClassFileField {
    pub flags: FieldFlags,
    pub name: String,
    pub type_descriptor: String,
    pub constant_value: Option<FieldConstantValue>,
}

impl fmt::Display for ClassFileField {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} {}: {} constant {:?}",
            self.flags, self.name, self.type_descriptor, self.constant_value,
        )
    }
}

#[derive(Debug, PartialEq, strum_macros::Display)]
pub enum FieldConstantValue {
    Int(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    String(String),
}
