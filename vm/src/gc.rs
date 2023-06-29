use std::ptr::null;
use std::{alloc::Layout, fmt, fmt::Formatter, marker::PhantomData};

use log::{debug, info};

use rjvm_reader::field_type::FieldType;
use rjvm_utils::type_conversion::ToUsizeSafe;

use crate::abstract_object::ALLOC_HEADER_SIZE;
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

struct MemoryChunk {
    memory: *mut u8,
    used: usize,
    capacity: usize,
}

impl fmt::Debug for MemoryChunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{{used={}, capacity={}}}", self.used, self.capacity)
    }
}

impl MemoryChunk {
    fn new(capacity: usize) -> Self {
        let layout = Layout::from_size_align(capacity, 8).unwrap();
        let ptr = unsafe { std::alloc::alloc_zeroed(layout) };
        println!(
            "allocated memory chunk of size {} at {:#0x}",
            capacity, ptr as u64
        );

        MemoryChunk {
            memory: ptr,
            capacity,
            used: 0,
        }
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

    unsafe fn contains(&self, ptr: *const u8) -> bool {
        ptr >= self.memory && ptr <= self.memory.add(self.used)
    }
}

pub struct ObjectAllocator<'a> {
    current: MemoryChunk,
    other: MemoryChunk,
    marker: PhantomData<&'a AbstractObject<'a>>,
}

impl<'a> ObjectAllocator<'a> {
    pub fn with_maximum_memory(max_size: usize) -> Self {
        let semi_space_capacity = max_size / 2;
        Self {
            current: MemoryChunk::new(semi_space_capacity),
            other: MemoryChunk::new(semi_space_capacity),
            marker: Default::default(),
        }
    }

    pub fn allocate(&mut self, class: &Class<'a>) -> Option<AbstractObject<'a>> {
        let size = AbstractObject::size_of_object(class);
        self.current
            .alloc(size)
            .map(|alloc_entry| AbstractObject::new_object(class, alloc_entry))
    }

    pub fn allocate_array(
        &mut self,
        elements_type: ArrayEntryType,
        length: usize,
    ) -> Option<AbstractObject<'a>> {
        let size = AbstractObject::size_of_array(length);
        self.current
            .alloc(size)
            .map(|alloc_entry| AbstractObject::new_array(elements_type, length, &alloc_entry))
    }

    pub unsafe fn do_garbage_collection(
        &mut self,
        roots: Vec<*mut AbstractObject<'a>>,
        class_resolver: &impl ClassByIdResolver<'a>,
    ) -> Result<(), VmError> {
        info!(
            "running gc; currently allocated memory = {}, gc roots count: {}",
            self.current.used,
            roots.len()
        );

        // Copy all reachable objects to the other region
        for root in roots.iter() {
            self.visit(*root, class_resolver)?;
        }

        self.fix_references_in_new_region(class_resolver)?;

        for root in roots {
            self.fix_gc_root(root);
        }

        // Swap regions and reset alloc pointer
        std::mem::swap(&mut self.current, &mut self.other);
        info!(
            "gc done; previous allocated memory = {}, new allocated memory = {}",
            self.other.used, self.current.used
        );
        self.other.used = 0;

        Ok(())
    }

    unsafe fn visit(
        &mut self,
        object_ptr: *const AbstractObject<'a>,
        class_resolver: &impl ClassByIdResolver<'a>,
    ) -> Result<(), VmError> {
        let referred_object_ptr = *(object_ptr as *const *mut u8);
        assert!(self.current.contains(referred_object_ptr));
        let header = &mut *(referred_object_ptr as *mut AllocHeader);

        match header.state() {
            GcState::Unmarked => {
                // Set as in progress to avoid infinite loops on references cycles
                header.set_state(GcState::Marked);

                // Visit members (object fields or array entries)
                if header.kind() == ObjectKind::Object {
                    self.visit_fields_of_object(&*object_ptr, class_resolver)?;
                } else {
                    self.visit_entries_of_array(&*object_ptr, class_resolver)?;
                }

                // Copy to other region as-is (with pointers to the current region)
                let new_address = self
                    .other
                    .alloc(header.size())
                    .map(|alloc_entry| {
                        std::ptr::copy_nonoverlapping(
                            referred_object_ptr,
                            alloc_entry.ptr,
                            header.size(),
                        );
                        alloc_entry.ptr
                    })
                    .expect("should have enough space in the other region");

                // Replace content of this object with forward reference to the new object
                std::ptr::write(
                    referred_object_ptr.add(ALLOC_HEADER_SIZE) as *mut *mut u8,
                    new_address,
                );
            }

            GcState::Marked => {
                // Already visited
            }
        }

        Ok(())
    }

    unsafe fn visit_fields_of_object(
        &mut self,
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
            self.visit(field_object_ptr, class_resolver)?;
        }
        Ok(())
    }

    unsafe fn visit_entries_of_array(
        &mut self,
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
                            self.visit(&array_element as *const AbstractObject, class_resolver)?;
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

    unsafe fn fix_references_in_new_region(
        &mut self,
        class_resolver: &impl ClassByIdResolver<'a>,
    ) -> Result<(), VmError> {
        let end_ptr = self.other.memory.add(self.other.used);
        let mut ptr = self.other.memory;
        while ptr < end_ptr {
            let header = &mut *(ptr as *mut AllocHeader);
            let object = AbstractObject::from_raw_ptr(ptr);

            if header.kind() == ObjectKind::Object {
                self.fix_references_in_object(object, class_resolver)?;
            } else {
                self.fix_references_in_array(object)?;
            }

            header.set_state(GcState::Unmarked);
            ptr = ptr.add(header.size());
        }
        Ok(())
    }

    unsafe fn fix_references_in_object(
        &self,
        object: AbstractObject<'a>,
        class_resolver: &impl ClassByIdResolver<'a>,
    ) -> Result<(), VmError> {
        let class = class_resolver
            .find_class_by_id(object.class_id())
            .ok_or(VmError::ValidationException)?;

        debug!("fixing members of {:?} of class {}", object, class.name);

        for (index, field) in class.all_fields().enumerate().filter(|(_, f)| {
            matches!(
                f.type_descriptor,
                FieldType::Object(_) | FieldType::Array(_)
            )
        }) {
            let field_value_ptr = object.ptr_to_field_value(index);
            debug!(
                "  need to fix field {} at offset {:#0x}",
                field.name, field_value_ptr as u64
            );

            let new_address = self.fix_reference(field_value_ptr);
            debug!(
                "  fixed field {} at offset {:#0x} - new value is {:#0x}",
                field.name, field_value_ptr as u64, new_address as u64
            );
        }
        Ok(())
    }

    unsafe fn fix_references_in_array(&mut self, array: AbstractObject<'a>) -> Result<(), VmError> {
        match array.elements_type() {
            ArrayEntryType::Base(_) => {
                // No objects are kept alive by this GC-reachable array!
                Ok(())
            }
            ArrayEntryType::Object(class_id) => {
                debug!("fixing entries of array {:?} of type {}", array, class_id);
                for i in 0..array.len().into_usize_safe() {
                    let element_ptr = array.ptr_to_array_element(i);
                    debug!(
                        "  need to fix element {} at offset {:#0x}",
                        i, element_ptr as u64
                    );

                    let new_address = self.fix_reference(element_ptr);
                    debug!(
                        "  fixed element {} at offset {:#0x} - new value is {:#0x}",
                        i, element_ptr as u64, new_address as u64
                    );
                }
                Ok(())
            }
            ArrayEntryType::Array => {
                todo!("arrays of arrays are not supported yet")
            }
        }
    }

    unsafe fn fix_reference(&self, field_value_ptr: *mut u8) -> *const u8 {
        if 0 == std::ptr::read(field_value_ptr as *const u64) {
            // Skipping nulls
            return null();
        }

        // Write new address, stored in the word after the header in the old object
        let old_referred_object = std::ptr::read(field_value_ptr as *const *const u8);
        assert!(self.current.contains(old_referred_object));
        let new_referred_object_address =
            std::ptr::read(old_referred_object.add(ALLOC_HEADER_SIZE) as *const *const u8);
        assert!(self.other.contains(new_referred_object_address));
        std::ptr::write(
            field_value_ptr as *mut *const u8,
            new_referred_object_address,
        );
        new_referred_object_address
    }

    unsafe fn fix_gc_root(&self, root: *mut AbstractObject<'a>) {
        debug!("fixing gc root {:#0x}", root as u64);
        self.fix_reference(root as *mut u8);
        debug!("  fixed gc root - new pointer is {:#0x}", root as u64);
    }
}

impl<'a> fmt::Debug for ObjectAllocator<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{{current_space={:?}}}", self.current)
    }
}
