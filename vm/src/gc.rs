use std::{alloc::Layout, fmt, fmt::Formatter, marker::PhantomData};

use bitfield_struct::bitfield;

use crate::{array::Array, array_entry_type::ArrayEntryType, class::Class, object::Object};

pub struct ObjectAllocator<'a> {
    memory: *mut u8,
    used: usize,
    capacity: usize,
    marker: PhantomData<&'a Object<'a>>,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u8)]
enum GcState {
    Unmarked = 0,
    InProgress = 1,
    Marked = 2,
}

impl From<u32> for GcState {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Unmarked,
            1 => Self::InProgress,
            2 => Self::Marked,
            _ => panic!("invalid value for GcState: {}", value),
        }
    }
}

impl From<GcState> for u32 {
    fn from(value: GcState) -> Self {
        value as u32
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u8)]
enum AllocKind {
    Object = 0,
    Array = 1,
}

impl From<u32> for AllocKind {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Object,
            1 => Self::Array,
            _ => panic!("invalid value for GcState: {}", value),
        }
    }
}

impl From<AllocKind> for u32 {
    fn from(value: AllocKind) -> Self {
        value as u32
    }
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
struct Header {
    #[bits(1)]
    kind: AllocKind,

    #[bits(2)]
    state: GcState,

    #[bits(29)]
    size: usize,
}

const HEADER_SIZE: usize = std::mem::size_of::<Header>();

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
        let ptr = self.alloc(size, AllocKind::Object);
        Object::new(class, ptr)
    }

    pub fn allocate_array(&mut self, elements_type: ArrayEntryType, length: usize) -> Array<'a> {
        let size = Array::size(length);
        let ptr = self.alloc(size, AllocKind::Array);
        Array::new(elements_type, length, ptr)
    }

    fn alloc(&mut self, size: usize, kind: AllocKind) -> *mut u8 {
        if self.used + size > self.capacity {
            // TODO: trigger garbage collection!
            panic!("no more memory!")
        }

        let alloc_size = size + HEADER_SIZE;
        let ptr = unsafe { self.memory.add(self.used) };
        let header = Header::new()
            .with_kind(kind)
            .with_state(GcState::Unmarked)
            .with_size(size);
        unsafe {
            std::ptr::write(ptr as *mut Header, header);
        }

        self.used += alloc_size;
        unsafe { ptr.add(HEADER_SIZE) }
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
