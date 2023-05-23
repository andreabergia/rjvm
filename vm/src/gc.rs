use std::{alloc::Layout, fmt, fmt::Formatter, marker::PhantomData};

use crate::{class::Class, object::ObjectValue};

pub struct ObjectAllocator<'a> {
    memory: *mut u8,
    used: usize,
    capacity: usize,
    marker: PhantomData<&'a ObjectValue<'a>>,
}

impl<'a> ObjectAllocator<'a> {
    pub fn with_maximum_memory(max_size: usize) -> Self {
        let result = Layout::from_size_align(max_size, 8).unwrap();
        let memory = unsafe { std::alloc::alloc_zeroed(result) };
        Self {
            memory,
            used: 0,
            capacity: max_size,
            marker: Default::default(),
        }
    }

    pub fn allocate(&mut self, class: &Class<'a>) -> ObjectValue<'a> {
        let size = ObjectValue::size(class);
        let ptr = self.alloc(size);
        ObjectValue::new(class, ptr)
    }

    fn alloc(&mut self, size: usize) -> *mut u8 {
        if self.used + size > self.capacity {
            // TODO: trigger garbage collection!
            panic!("no more memory!")
        }
        let ptr = unsafe { self.memory.add(self.used) };
        self.used += size;
        ptr
    }
}

impl<'a> fmt::Debug for ObjectAllocator<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "object_allocator={{used={}, capacity={}}}",
            self.used, self.capacity
        )
    }
}
