use std::fmt;
use std::fmt::Formatter;

use result::prelude::*;
use typed_arena::Arena;

use rjvm_reader::class_file::ClassFile;

use crate::{class::Class, vm_error::VmError};

pub trait ClassResolver<'a> {
    fn find_class(&self, name: &str) -> Option<&'a Class<'a>>;
}

pub struct ClassAllocator<'a> {
    arena: Arena<Class<'a>>,
}

impl<'a> Default for ClassAllocator<'a> {
    fn default() -> Self {
        Self {
            arena: Arena::with_capacity(100),
        }
    }
}

impl<'a> fmt::Debug for ClassAllocator<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "class_allocator={{len={}}}", self.arena.len())
    }
}

impl<'a> ClassAllocator<'a> {
    pub fn allocate<'b>(
        &'b self,
        class_file: ClassFile,
        resolver: &impl ClassResolver<'a>,
    ) -> Result<&'a Class<'a>, VmError> {
        let class = Self::new_class(class_file, resolver)?;
        let class_ref = self.arena.alloc(class);

        // SAFETY: our reference class_ref is alive only for 'b.
        // However we actually know that the arena will keep the value alive for 'a,
        // and I cannot find a way to convince the compiler of this fact. Thus
        // we use this pointer "trick" to make the compiler happy
        unsafe {
            let class_ptr: *const Class<'a> = class_ref;
            Ok(&*class_ptr)
        }
    }

    fn new_class(
        class_file: ClassFile,
        resolver: &impl ClassResolver<'a>,
    ) -> Result<Class<'a>, VmError> {
        let superclass = class_file
            .superclass
            .as_ref()
            .map(|superclass_name| {
                resolver
                    .find_class(superclass_name)
                    .ok_or(VmError::ClassNotFoundException(superclass_name.clone()))
            })
            .invert()?;
        let interfaces: Result<Vec<&Class>, VmError> = class_file
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
