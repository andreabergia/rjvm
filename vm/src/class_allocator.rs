use std::{fmt, fmt::Formatter};

use result::prelude::*;
use typed_arena::Arena;

use rjvm_reader::class_file::ClassFile;

use crate::{
    class::{Class, ClassId, ClassRef},
    vm_error::VmError,
};

pub trait ClassResolver<'a> {
    fn find_class_by_id(&self, id: ClassId) -> Option<ClassRef<'a>>;
    fn find_class_by_name(&self, name: &str) -> Option<ClassRef<'a>>;
}

pub struct ClassAllocator<'a> {
    arena: Arena<Class<'a>>,
    next_id: u64,
}

impl<'a> Default for ClassAllocator<'a> {
    fn default() -> Self {
        Self {
            arena: Arena::with_capacity(100),
            next_id: 1,
        }
    }
}

impl<'a> fmt::Debug for ClassAllocator<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "class_allocator={{len={}}}", self.arena.len())
    }
}

impl<'a> ClassAllocator<'a> {
    pub fn allocate(
        &mut self,
        class_file: ClassFile,
        resolver: &impl ClassResolver<'a>,
    ) -> Result<ClassRef<'a>, VmError> {
        let next_id = self.next_id;
        self.next_id += 1;

        let class = Self::new_class(class_file, ClassId::new(next_id), resolver)?;
        let class_ref = self.arena.alloc(class);

        // SAFETY: our reference class_ref is alive only for 'b.
        // However we actually know that the arena will keep the value alive for 'a,
        // and I cannot find a way to convince the compiler of this fact. Thus
        // I'm using this pointer "trick" to make the compiler happy.
        // I'm sure this can be done with safe Rust, I just do not know how at the moment...
        unsafe {
            let class_ptr: *const Class<'a> = class_ref;
            Ok(&*class_ptr)
        }
    }

    fn new_class(
        class_file: ClassFile,
        id: ClassId,
        resolver: &impl ClassResolver<'a>,
    ) -> Result<Class<'a>, VmError> {
        let superclass = class_file
            .superclass
            .as_ref()
            .map(|superclass_name| {
                resolver
                    .find_class_by_name(superclass_name)
                    .ok_or(VmError::ClassNotFoundException(superclass_name.clone()))
            })
            .invert()?;
        let interfaces: Result<Vec<&Class>, VmError> = class_file
            .interfaces
            .iter()
            .map(|interface_name| {
                resolver
                    .find_class_by_name(interface_name)
                    .ok_or(VmError::ClassNotFoundException(interface_name.clone()))
            })
            .collect();

        let num_superclass_fields = match superclass {
            Some(superclass) => superclass.num_total_fields,
            None => 0,
        };
        let num_this_class_fields = class_file.fields.len();

        Ok(Class {
            id,
            name: class_file.name,
            constants: class_file.constants,
            flags: class_file.flags,
            superclass,
            interfaces: interfaces?,
            fields: class_file.fields,
            methods: class_file.methods,
            num_total_fields: num_superclass_fields + num_this_class_fields,
            first_field_index: num_superclass_fields,
        })
    }
}
