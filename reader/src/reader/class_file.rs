use std::fmt;
use std::rc::Rc;

use crate::reader::{
    class_access_flags::ClassAccessFlags, class_file_field::ClassFileField,
    class_file_method::ClassFileMethod, class_file_version::ClassFileVersion,
    constant_pool::ConstantPool,
};
use crate::vm::vm_error::VmError;

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

    pub fn get_method(
        &self,
        method_name: &str,
        type_descriptor: &str,
    ) -> Result<Rc<ClassFileMethod>, VmError> {
        self.find_method(method_name, type_descriptor)
            .ok_or(VmError::MethodNotFoundException(
                self.name.to_string(),
                method_name.to_string(),
                type_descriptor.to_string(),
            ))
    }

    pub fn find_field(&self, field_name: &str) -> Option<(usize, &ClassFileField)> {
        // TODO: replace linear search with something faster
        self.fields
            .iter()
            .enumerate()
            .find(|entry| entry.1.name == field_name)
    }

    pub fn get_field(&self, field_name: &str) -> Result<(usize, &ClassFileField), VmError> {
        self.find_field(field_name)
            .ok_or(VmError::FieldNotFoundException(
                self.name.to_string(),
                field_name.to_string(),
            ))
    }
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
            writeln!(f, "  - {field}")?;
        }
        writeln!(f, "methods:")?;
        for method in self.methods.iter() {
            writeln!(f, "  - {method}")?;
        }
        Ok(())
    }
}
