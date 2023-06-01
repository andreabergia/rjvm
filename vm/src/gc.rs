use std::{alloc::Layout, fmt, fmt::Formatter, marker::PhantomData};

use crate::{array::Array, array_entry_type::ArrayEntryType, class::Class, object::Object};

pub struct ObjectAllocator<'a> {
    memory: *mut u8,
    used: usize,
    capacity: usize,
    marker: PhantomData<&'a Object<'a>>,
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

    pub fn allocate(&mut self, class: &Class<'a>) -> Object<'a> {
        let size = Object::size(class);
        let ptr = self.alloc(size);
        Object::new(class, ptr)
    }

    pub fn allocate_array(&mut self, elements_type: ArrayEntryType, length: usize) -> Array<'a> {
        let size = Array::size(length);
        let ptr = self.alloc(size);
        Array::new(elements_type, length, ptr)
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
