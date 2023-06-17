use std::{alloc::Layout, fmt, fmt::Formatter, marker::PhantomData, ptr::NonNull};

use bitfield_struct::bitfield;
use log::debug;

use rjvm_reader::field_type::FieldType;

use crate::{
    array::Array, array_entry_type::ArrayEntryType, class::Class,
    class_resolver_by_id::ClassByIdResolver, object::Object,
};

pub struct ObjectAllocator<'a> {
    memory: *mut u8,
    used: usize,
    capacity: usize,
    marker: PhantomData<&'a Object<'a>>,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum GcState {
    Unmarked,
    InProgress,
    Marked,
}

impl From<u64> for GcState {
    fn from(value: u64) -> Self {
        match value {
            0 => Self::Unmarked,
            1 => Self::InProgress,
            2 => Self::Marked,
            _ => panic!("invalid value for GcState: {}", value),
        }
    }
}

impl From<GcState> for u64 {
    fn from(value: GcState) -> Self {
        value as u64
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum AllocKind {
    Object,
    Array,
}

impl From<u64> for AllocKind {
    fn from(value: u64) -> Self {
        match value {
            0 => Self::Object,
            1 => Self::Array,
            _ => panic!("invalid value for GcState: {}", value),
        }
    }
}

impl From<AllocKind> for u64 {
    fn from(value: AllocKind) -> Self {
        value as u64
    }
}

#[bitfield(u64)]
#[derive(PartialEq, Eq)]
struct Header {
    #[bits(1)]
    kind: AllocKind,

    #[bits(2)]
    state: GcState,

    #[bits(61)]
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

    pub fn allocate(&mut self, class: &Class<'a>) -> Option<Object<'a>> {
        let size = Object::size(class);
        self.alloc(size, AllocKind::Object)
            .map(|ptr| Object::new(class, ptr.as_ptr()))
    }

    pub fn allocate_array(
        &mut self,
        elements_type: ArrayEntryType,
        length: usize,
    ) -> Option<Array<'a>> {
        let size = Array::size(length);
        self.alloc(size, AllocKind::Array)
            .map(|ptr| Array::new(elements_type, length, ptr.as_ptr()))
    }

    fn alloc(&mut self, size: usize, kind: AllocKind) -> Option<NonNull<u8>> {
        if self.used + size + HEADER_SIZE > self.capacity {
            return None;
        }

        // Align to 8 bytes
        let alloc_size = size + HEADER_SIZE;
        let alloc_size = match alloc_size % 8 {
            0 => alloc_size,
            n => alloc_size + (8 - n),
        };

        let ptr = unsafe { self.memory.add(self.used) };
        self.used += alloc_size;

        let header = Header::new()
            .with_kind(kind)
            .with_state(GcState::Unmarked)
            .with_size(alloc_size);
        unsafe {
            std::ptr::write(ptr as *mut Header, header);
            NonNull::new(ptr.add(HEADER_SIZE))
        }
    }

    pub unsafe fn do_garbage_collection(&mut self, roots: Vec<*mut Object<'a>>) {
        self.unmark_all_objects();

        // Mark all reachable objects
        for root in roots {
            self.mark(root);
        }

        self.log_marked_objects_for_debug();
    }

    unsafe fn unmark_all_objects(&mut self) {
        let end_ptr = self.memory.add(self.used);
        let mut ptr = self.memory;
        while ptr < end_ptr {
            let header = &mut *(ptr as *mut Header);
            header.set_state(GcState::Unmarked);
            ptr = ptr.add(header.size());
        }
    }

    unsafe fn mark(&self, object_ptr: *mut Object<'a>) {
        let referred_object_ptr = *(object_ptr as *const *mut u8);
        assert!(
            referred_object_ptr >= self.memory && referred_object_ptr <= self.memory.add(self.used)
        );
        let header_location = referred_object_ptr.offset(-(HEADER_SIZE as isize));
        let header = &mut *(header_location as *mut Header);

        match header.state() {
            GcState::Unmarked => {
                header.set_state(GcState::InProgress);
                self.visit_members_of(&*object_ptr);
                header.set_state(GcState::Marked);
            }

            GcState::InProgress | GcState::Marked => {
                // Already visited
            }
        }
    }

    unsafe fn visit_members_of(&self, object: &Object<'a>) {
        // TODO
        debug!("should visit members of {:?}", object);
    }

    // TODO: remove
    unsafe fn log_marked_objects_for_debug(&mut self) {
        let end_ptr = self.memory.add(self.used);
        let mut ptr = self.memory;
        while ptr < end_ptr {
            let header = &mut *(ptr as *mut Header);
            if header.state() == GcState::Marked {
                debug!("marked object: {:?}", ptr);
            } else {
                debug!("unmarked object: {:?}", ptr);
            }
            ptr = ptr.add(header.size());
        }
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
