use std::{alloc::Layout, fmt, fmt::Formatter, marker::PhantomData};

use log::debug;

use rjvm_reader::field_type::FieldType;
use rjvm_utils::type_conversion::ToUsizeSafe;

use crate::{
    abstract_object::{AbstractObject, AllocHeader, GcState, ObjectKind},
    alloc_entry::AllocEntry,
    array::Array,
    array_entry_type::ArrayEntryType,
    class::Class,
    class_resolver_by_id::ClassByIdResolver,
    object::Object,
    value::Value,
    vm_error::VmError,
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
            .map(|alloc_entry| AbstractObject::new_object(class, alloc_entry))
    }

    pub fn allocate_array(
        &mut self,
        elements_type: ArrayEntryType,
        length: usize,
    ) -> Option<AbstractObject<'a>> {
        let size = AbstractObject::size_of_array(length);
        self.alloc(size)
            .map(|alloc_entry| AbstractObject::new_array(elements_type, length, &alloc_entry))
    }

    fn alloc(&mut self, required_size: usize) -> Option<AllocEntry> {
        if self.used + required_size > self.capacity {
            return None;
        }

        // We need all allocations to be aligned to 8
        assert_eq!(required_size % 8, 0);

        let ptr = unsafe { self.memory.add(self.used) };
        self.used += required_size;

        Some(AllocEntry {
            ptr,
            alloc_size: required_size,
        })
    }

    pub unsafe fn do_garbage_collection(
        &mut self,
        roots: Vec<*mut AbstractObject<'a>>,
        class_resolver: &impl ClassByIdResolver<'a>,
    ) -> Result<(), VmError> {
        self.unmark_all_objects();

        // Mark all reachable objects
        for root in roots {
            self.mark(root, class_resolver)?;
        }

        self.log_marked_objects_for_debug();
        Ok(())
    }

    unsafe fn unmark_all_objects(&mut self) {
        let end_ptr = self.memory.add(self.used);
        let mut ptr = self.memory;
        while ptr < end_ptr {
            let header = &mut *(ptr as *mut AllocHeader);
            header.set_state(GcState::Unmarked);
            ptr = ptr.add(header.size());
        }
    }

    unsafe fn mark(
        &self,
        object_ptr: *const AbstractObject<'a>,
        class_resolver: &impl ClassByIdResolver<'a>,
    ) -> Result<(), VmError> {
        let referred_object_ptr = *(object_ptr as *const *mut u8);
        assert!(
            referred_object_ptr >= self.memory && referred_object_ptr <= self.memory.add(self.used)
        );
        let header = &mut *(referred_object_ptr as *mut AllocHeader);

        match header.state() {
            GcState::Unmarked => {
                header.set_state(GcState::InProgress);
                if header.kind() == ObjectKind::Object {
                    self.mark_members_of_object(&*object_ptr, class_resolver)?;
                } else {
                    self.visit_entries_of_array(&*object_ptr, class_resolver)?;
                }
                header.set_state(GcState::Marked);
            }

            GcState::InProgress | GcState::Marked => {
                // Already visited
            }
        }

        Ok(())
    }

    unsafe fn mark_members_of_object(
        &self,
        object: &AbstractObject<'a>,
        class_resolver: &impl ClassByIdResolver<'a>,
    ) -> Result<(), VmError> {
        let class = class_resolver
            .find_class_by_id(object.class_id())
            .ok_or(VmError::ValidationException)?;

        debug!(
            "should visit members of {:?} of class {}",
            object, class.name
        );

        for (index, field) in class.all_fields().enumerate().filter(|(_, f)| {
            matches!(
                f.type_descriptor,
                FieldType::Object(_) | FieldType::Array(_)
            )
        }) {
            let field_value_ptr = object.ptr_to_field_value(index);
            debug!(
                "  should visit recursively field {} at offset {:#0x}",
                field.name, field_value_ptr as u64
            );

            if 0 == std::ptr::read(field_value_ptr as *const u64) {
                // Skipping nulls
                continue;
            }
            let field_object_ptr = field_value_ptr as *mut AbstractObject;
            self.mark(field_object_ptr, class_resolver)?;
        }
        Ok(())
    }

    unsafe fn visit_entries_of_array(
        &self,
        array: &AbstractObject<'a>,
        class_resolver: &impl ClassByIdResolver<'a>,
    ) -> Result<(), VmError> {
        match array.elements_type() {
            ArrayEntryType::Base(_) => {
                // No objects are kept alive by this GC-reachable array!
                Ok(())
            }
            ArrayEntryType::Object(_) => {
                for i in 0..array.len().into_usize_safe() {
                    let value = array.get_element(i);
                    match value {
                        Ok(Value::Object(array_element)) => {
                            debug!("  should visit recursively element at index {}", i);
                            self.mark(&array_element as *const AbstractObject, class_resolver)?;
                        }
                        Ok(Value::Null) => {
                            // Ok, skip it
                        }
                        _ => return Err(VmError::ValidationException),
                    }
                }
                Ok(())
            }
            ArrayEntryType::Array => {
                todo!("arrays of arrays are not supported yet")
            }
        }
    }

    // TODO: remove
    unsafe fn log_marked_objects_for_debug(&mut self) {
        let end_ptr = self.memory.add(self.used);
        let mut ptr = self.memory;
        while ptr < end_ptr {
            let header = &*(ptr as *const AllocHeader);
            let object = AbstractObject::from_raw_ptr(ptr);
            if header.state() == GcState::Marked {
                debug!("marked object: {:?} {:?}", ptr, object);
            } else {
                debug!("unmarked object: {:?} {:?}", ptr, object);
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
