use std::{alloc::Layout, fmt, fmt::Formatter, marker::PhantomData, ptr::NonNull};

use crate::abstract_object::AbstractObject;
use crate::{
    array_entry_type::ArrayEntryType, class::Class, class_resolver_by_id::ClassByIdResolver,
};

pub struct ObjectAllocator<'a> {
    memory: *mut u8,
    used: usize,
    capacity: usize,
    marker: PhantomData<&'a AbstractObject<'a>>,
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

    pub fn allocate(&mut self, class: &Class<'a>) -> Option<AbstractObject<'a>> {
        let size = AbstractObject::size_of_object(class);
        self.alloc(size)
            .map(|ptr| AbstractObject::new_object(class, ptr.as_ptr()))
    }

    pub fn allocate_array(
        &mut self,
        elements_type: ArrayEntryType,
        length: usize,
    ) -> Option<AbstractObject<'a>> {
        let size = AbstractObject::size_of_array(length);
        self.alloc(size)
            .map(|ptr| AbstractObject::new_array(elements_type, length, ptr.as_ptr()))
    }

    fn alloc(&mut self, alloc_size: usize) -> Option<NonNull<u8>> {
        if self.used + alloc_size > self.capacity {
            return None;
        }

        // Align to 8 bytes
        let alloc_size = match alloc_size % 8 {
            0 => alloc_size,
            n => alloc_size + (8 - n),
        };

        let ptr = unsafe { self.memory.add(self.used) };
        self.used += alloc_size;

        NonNull::new(ptr)
    }

    pub unsafe fn do_garbage_collection(
        &mut self,
        _roots: Vec<*mut AbstractObject<'a>>,
        _class_resolver: &impl ClassByIdResolver<'a>,
    ) {
        todo!("Implement GC");
        /*  self.unmark_all_objects();

            // Mark all reachable objects
            for root in roots {
                self.mark(root, class_resolver);
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

        unsafe fn mark(
            &self,
            object_ptr: *mut Object<'a>,
            class_resolver: &impl ClassByIdResolver<'a>,
        ) {
            let referred_object_ptr = *(object_ptr as *const *mut u8);
            assert!(
                referred_object_ptr >= self.memory && referred_object_ptr <= self.memory.add(self.used)
            );
            let header_location = referred_object_ptr.offset(-(HEADER_SIZE as isize));
            let header = &mut *(header_location as *mut Header);

            match header.state() {
                GcState::Unmarked => {
                    header.set_state(GcState::InProgress);
                    self.visit_members_of(&*object_ptr, class_resolver);
                    header.set_state(GcState::Marked);
                }

                GcState::InProgress | GcState::Marked => {
                    // Already visited
                }
            }
        }

        unsafe fn visit_members_of(
            &self,
            object: &Object<'a>,
            class_resolver: &impl ClassByIdResolver<'a>,
        ) {
            // TODO: return an error?
            let class = class_resolver
                .find_class_by_id(object.class_id())
                .expect("objects should have a valid class reference");

            debug!(
                "should visit members of {:?} of class {}",
                object, class.name
            );

            class
                .all_fields()
                .enumerate()
                .filter(|(_, f)| {
                    matches!(
                        f.type_descriptor,
                        FieldType::Object(_) // TODO: add arrays
                                             //  | FieldType::Array(_)
                    )
                })
                .for_each(|(index, field)| {
                    debug!(
                        "  should visit recursively field {} of object {:?}",
                        field.name, object
                    );

                    let field_value_ptr = object.offset_of_field(index);
                    if 0 == std::ptr::read(field_value_ptr as *const u64) {
                        // Skipping nulls
                        return;
                    }
                    let field_object_ptr = field_value_ptr as *mut Object;
                    self.mark(field_object_ptr, class_resolver);
                })
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
            }*/
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
