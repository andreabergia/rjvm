use std::fmt;

use crate::class_file_field::ClassFileField;
use crate::{
    class_access_flags::ClassAccessFlags, class_file_version::ClassFileVersion,
    constant_pool::ConstantPool,
};

/// Represents the content of a .class file.
#[derive(Debug, Default)]
pub struct ClassFile {
    pub version: ClassFileVersion,
    pub constants: ConstantPool,
    pub flags: ClassAccessFlags,
    pub name: String,
    pub superclass: String,
    pub interfaces: Vec<String>,
    pub fields: Vec<ClassFileField>,
}

impl fmt::Display for ClassFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "Class {} (extends {}), version: {}",
            self.name, self.superclass, self.version
        )?;
        write!(f, "{}", self.constants)?;
        writeln!(f, "flags: {:?}", self.flags)?;
        writeln!(f, "interfaces: {:?}", self.interfaces)?;
        writeln!(f, "fields:")?;
        for field in self.fields.iter() {
            writeln!(f, "  - {}", field)?;
        }
        Ok(())
    }
}
