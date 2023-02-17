use std::rc::Rc;
use std::sync::Arc;

use result::prelude::*;

use rjvm_reader::{
    class_access_flags::ClassAccessFlags, class_file::ClassFile, class_file_field::ClassFileField,
    class_file_method::ClassFileMethod, constant_pool::ConstantPool,
};

use crate::vm_error::VmError;

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub constants: ConstantPool,
    pub flags: ClassAccessFlags,
    pub superclass: Option<Arc<Class>>,
    pub interfaces: Vec<Arc<Class>>,
    pub fields: Vec<ClassFileField>,
    pub methods: Vec<Rc<ClassFileMethod>>,
}

pub trait ClassResolver {
    fn find_class(&self, name: &str) -> Option<Arc<Class>>;
}

impl Class {
    pub fn new(class_file: ClassFile, resolver: &impl ClassResolver) -> Result<Class, VmError> {
        let superclass = class_file
            .superclass
            .as_ref()
            .map(|superclass_name| {
                resolver
                    .find_class(superclass_name)
                    .ok_or(VmError::ClassNotFoundException(superclass_name.clone()))
            })
            .invert()?;
        let interfaces: Result<Vec<Arc<Class>>, VmError> = class_file
            .interfaces
            .iter()
            .map(|interface_name| {
                resolver
                    .find_class(interface_name)
                    .ok_or(VmError::ClassNotFoundException(interface_name.clone()))
            })
            .collect();

        Ok(Class {
            name: class_file.name,
            constants: class_file.constants,
            flags: class_file.flags,
            superclass,
            interfaces: interfaces?,
            fields: class_file.fields,
            methods: class_file.methods,
        })
    }

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
