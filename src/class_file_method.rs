use std::fmt;
use std::fmt::Formatter;

use crate::attribute::Attribute;
use crate::method_flags::MethodFlags;

#[derive(Debug, Default, PartialEq)]
pub struct ClassFileMethod {
    pub flags: MethodFlags,
    pub name: String,
    pub type_descriptor: String,
    pub attributes: Vec<Attribute>,
}

impl fmt::Display for ClassFileMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} {}: {} ({:?})",
            self.flags, self.name, self.type_descriptor, self.attributes,
        )
    }
}
