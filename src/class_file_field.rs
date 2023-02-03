use std::fmt;
use std::fmt::Formatter;

use crate::attribute::Attribute;
use crate::field_flags::FieldFlags;

#[derive(Debug, Default, PartialEq)]
pub struct ClassFileField {
    pub flags: FieldFlags,
    pub name: String,
    pub type_descriptor: String,
    pub attributes: Vec<Attribute>,
}

impl fmt::Display for ClassFileField {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} {}: {} ({:?})",
            self.flags, self.name, self.type_descriptor, self.attributes,
        )
    }
}
