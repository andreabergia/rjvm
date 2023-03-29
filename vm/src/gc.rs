use std::{fmt, fmt::Formatter, marker::PhantomData};

use typed_arena::Arena;

use crate::{
    class::Class,
    value::{ObjectRef, ObjectValue},
};

pub struct ObjectAllocator<'a> {
    arena: Arena<ObjectValue<'a>>,
    marker: PhantomData<&'a ObjectValue<'a>>,
}

impl<'a> Default for ObjectAllocator<'a> {
    fn default() -> Self {
        Self {
            arena: Arena::with_capacity(1000),
            marker: Default::default(),
        }
    }
}

impl<'a> fmt::Debug for ObjectAllocator<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "object_allocator={{len={}}}", self.arena.len())
    }
}

impl<'a> ObjectAllocator<'a> {
    pub fn allocate(&mut self, class: &Class<'a>) -> ObjectRef<'a> {
        let new_object = self.arena.alloc(ObjectValue::new(class));

        // SAFETY: same as ClassAllocator
        unsafe {
            let object_ptr: *const ObjectValue = new_object;
            &*object_ptr
        }
    }
}
