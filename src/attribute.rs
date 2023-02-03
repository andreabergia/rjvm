use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Default, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub info: Vec<u8>,
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} (data = {} bytes)", self.name, self.info.len())
    }
}
