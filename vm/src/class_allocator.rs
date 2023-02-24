use std::fmt;
use std::fmt::Formatter;

use result::prelude::*;
use typed_arena::Arena;

use rjvm_reader::class_file::ClassFile;

use crate::class::{Class, ClassPtr};
use crate::vm_error::VmError;

pub trait ClassResolver {
    fn find_class(&self, name: &str) -> Option<ClassPtr>;
}

pub struct ClassAllocator {
    arena: Arena<Class>,
}

impl Default for ClassAllocator {
    fn default() -> Self {
        Self {
            arena: Arena::with_capacity(100),
        }
    }
}

impl fmt::Debug for ClassAllocator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "class_allocator={{len={}}}", self.arena.len())
    }
}

impl ClassAllocator {
    pub fn allocate(
        &mut self,
        class_file: ClassFile,
        resolver: &impl ClassResolver,
    ) -> Result<ClassPtr, VmError> {
        let class = Self::new_class(class_file, resolver)?;
        let class_ref = self.arena.alloc(class);
        Ok(ClassPtr::new(class_ref))
    }

    fn new_class(class_file: ClassFile, resolver: &impl ClassResolver) -> Result<Class, VmError> {
        let superclass = class_file
            .superclass
            .as_ref()
            .map(|superclass_name| {
                resolver
                    .find_class(superclass_name)
                    .ok_or(VmError::ClassNotFoundException(superclass_name.clone()))
            })
            .invert()?;
        let interfaces: Result<Vec<ClassPtr>, VmError> = class_file
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
}
