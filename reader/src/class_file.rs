use std::fmt;
use std::rc::Rc;

use crate::{
    class_access_flags::ClassAccessFlags, class_file_field::ClassFileField,
    class_file_method::ClassFileMethod, class_file_version::ClassFileVersion,
    constant_pool::ConstantPool,
};

/// Represents the content of a .class file.
#[derive(Debug, Default)]
pub struct ClassFile {
    pub version: ClassFileVersion,
    pub constants: ConstantPool,
    pub flags: ClassAccessFlags,
    pub name: String,
    pub superclass: Option<String>,
    pub interfaces: Vec<String>,
    pub fields: Vec<ClassFileField>,
    pub methods: Vec<Rc<ClassFileMethod>>,
}

impl ClassFile {
    pub fn find_method(
        &self,
        method_name: &str,
        type_descriptor: &str,
    ) -> Option<Rc<ClassFileMethod>> {
        // TODO: replace linear search with something faster
        self.methods
            .iter()
            .find(|method| method.name == method_name && method.type_descriptor == type_descriptor)
            .cloned()
    }

    pub fn find_field(&self, field_name: &str) -> Option<(usize, &ClassFileField)> {
        // TODO: replace linear search with something faster
        self.fields
            .iter()
            .enumerate()
            .find(|entry| entry.1.name == field_name)
    }
}

impl fmt::Display for ClassFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Class {} ", self.name,)?;
        if let Some(superclass) = self.superclass.as_ref() {
            write!(f, "(extends {}) ", superclass)?;
        }
        writeln!(f, "version: {}", self.version)?;
        write!(f, "{}", self.constants)?;
        writeln!(f, "flags: {:?}", self.flags)?;
        writeln!(f, "interfaces: {:?}", self.interfaces)?;
        writeln!(f, "fields:")?;
        for field in self.fields.iter() {
            writeln!(f, "  - {field}")?;
        }
        writeln!(f, "methods:")?;
        for method in self.methods.iter() {
            writeln!(f, "  - {method}")?;
        }
        Ok(())
    }
}
