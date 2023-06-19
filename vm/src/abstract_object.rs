use std::{
    fmt::{Debug, Formatter},
    marker::PhantomData,
};

use bitfield_struct::bitfield;

use rjvm_reader::field_type::{BaseType, FieldType};
use rjvm_utils::type_conversion::ToUsizeSafe;

use crate::{
    array::Array,
    array_entry_type::ArrayEntryType,
    class::{Class, ClassId, ClassRef},
    object::Object,
    value::Value,
    vm_error::VmError,
};

#[derive(PartialEq, Clone)]
#[repr(transparent)]
pub struct AbstractObject<'a> {
    data: *mut u8,
    marker: PhantomData<&'a [u8]>,
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
pub enum ObjectKind {
    Object,
    Array,
}

impl From<u64> for ObjectKind {
    fn from(value: u64) -> Self {
        match value {
            0 => Self::Object,
            1 => Self::Array,
            _ => panic!("invalid value for GcState: {}", value),
        }
    }
}

impl From<ObjectKind> for u64 {
    fn from(value: ObjectKind) -> Self {
        value as u64
    }
}

#[bitfield(u64)]
#[derive(PartialEq, Eq)]
struct AllocHeader {
    #[bits(1)]
    kind: ObjectKind,

    #[bits(2)]
    state: GcState,

    #[bits(29)]
    identity_hash_code: i32,

    #[bits(32)]
    size: usize,
}

#[repr(transparent)]
struct ObjectHeader {
    class_id: ClassId,
}

struct ArrayHeader {
    elements_type: ArrayEntryType,
    length: u32,
}

const ALLOC_HEADER_SIZE: usize = std::mem::size_of::<AllocHeader>();
const OBJECT_HEADER_SIZE: usize = std::mem::size_of::<ObjectHeader>();
const ARRAY_HEADER_SIZE: usize = std::mem::size_of::<ArrayHeader>();

impl<'a> AbstractObject<'a> {
    // Each field will be stored in 8 bytes. This means we waste some memory
    // for fields that would fit in 4 or less, but it means computing a
    // field offset is trivial (index * 8) and that we have no problem with
    // memory alignment.
    pub(crate) fn size_of_object(class: &Class) -> usize {
        let fields_sizes: usize = 8 * class.num_total_fields;
        ALLOC_HEADER_SIZE + OBJECT_HEADER_SIZE + fields_sizes
    }

    // Similarly to objects, we waste some memory in exchange for simplicity.
    pub(crate) fn size_of_array(length: usize) -> usize {
        ALLOC_HEADER_SIZE + ARRAY_HEADER_SIZE + length * 8
    }

    pub fn new_object(class: &Class<'a>, ptr: *mut u8) -> Self {
        Self::write_object_header(class, ptr);
        Self {
            data: ptr,
            marker: PhantomData,
        }
    }

    fn write_object_header(class: &Class, ptr: *mut u8) {
        let alloc_size = Self::size_of_object(class);
        unsafe {
            let next_ptr = Self::write_alloc_header(ptr, alloc_size, ObjectKind::Object);
            std::ptr::write(
                next_ptr as *mut ObjectHeader,
                ObjectHeader { class_id: class.id },
            );
        }
    }

    pub fn new_array(elements_type: ArrayEntryType, array_length: usize, ptr: *mut u8) -> Self {
        Self::write_array_header(elements_type, array_length, ptr);
        Self {
            data: ptr,
            marker: PhantomData,
        }
    }

    fn write_array_header(elements_type: ArrayEntryType, array_length: usize, ptr: *mut u8) {
        let alloc_size = Self::size_of_array(array_length);
        unsafe {
            let next_ptr = Self::write_alloc_header(ptr, alloc_size, ObjectKind::Array);
            std::ptr::write(
                next_ptr as *mut ArrayHeader,
                ArrayHeader {
                    elements_type,
                    length: array_length as u32,
                },
            );
        }
    }

    unsafe fn write_alloc_header(ptr: *mut u8, alloc_size: usize, kind: ObjectKind) -> *mut u8 {
        let next_ptr = ptr as *mut AllocHeader;
        std::ptr::write(
            next_ptr,
            AllocHeader::new()
                .with_kind(kind)
                .with_state(GcState::Unmarked)
                .with_identity_hash_code(identity_hash_code(ptr))
                .with_size(alloc_size),
        );
        next_ptr.add(1) as *mut u8
    }

    // TODO: should we implement eq rather than this function?
    pub fn is_same_as(&self, other: &AbstractObject) -> bool {
        self.data == other.data
    }

    fn alloc_header(&self) -> &AllocHeader {
        unsafe { &*(self.data as *const AllocHeader) }
    }

    pub fn identity_hash_code(&self) -> i32 {
        self.alloc_header().identity_hash_code()
    }

    pub fn kind(&self) -> ObjectKind {
        self.alloc_header().kind()
    }

    pub fn alloc_size(&self) -> usize {
        self.alloc_header().size()
    }
}

impl<'a> Debug for AbstractObject<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} ptr {:#0x} size {}",
            self.kind(),
            self.data as usize,
            self.alloc_size(),
        )?;
        match self.kind() {
            ObjectKind::Object => write!(f, " class_id {}", self.class_id()),
            ObjectKind::Array => write!(
                f,
                " elements type {:?} len {}",
                self.elements_type(),
                self.len()
            ),
        }
    }
}

fn identity_hash_code(ptr: *mut u8) -> i32 {
    let addr = ptr as u64;
    let hash = (addr >> 32) ^ (addr);

    // Note: we'll take the 29 least significant bits here, since we'll store this in AllocHeader!
    let hash = (hash & ((1 << 29) - 1)) as u32;

    unsafe { std::mem::transmute(hash) }
}

unsafe fn write_value(ptr: *mut u8, value: Value) {
    match value {
        Value::Int(int) => std::ptr::write(ptr as *mut i32, int),
        Value::Long(long) => std::ptr::write(ptr as *mut i64, long),
        Value::Float(float) => std::ptr::write(ptr as *mut f32, float),
        Value::Double(double) => std::ptr::write(ptr as *mut f64, double),
        Value::Uninitialized | Value::Null => std::ptr::write(ptr as *mut u64, 0),
        Value::Object(obj) => std::ptr::write(ptr as *mut AbstractObject, obj),
    }
}

unsafe fn read_value<'a>(ptr: *const u8, field_type: &FieldType) -> Value<'a> {
    match field_type {
        FieldType::Base(BaseType::Boolean)
        | FieldType::Base(BaseType::Byte)
        | FieldType::Base(BaseType::Char)
        | FieldType::Base(BaseType::Short)
        | FieldType::Base(BaseType::Int) => Value::Int(std::ptr::read(ptr as *const i32)),
        FieldType::Base(BaseType::Long) => Value::Long(std::ptr::read(ptr as *const i64)),
        FieldType::Base(BaseType::Float) => Value::Float(std::ptr::read(ptr as *const f32)),
        FieldType::Base(BaseType::Double) => Value::Double(std::ptr::read(ptr as *const f64)),
        FieldType::Object(_) | FieldType::Array(_) => match std::ptr::read(ptr as *const i64) {
            0 => Value::Null,
            _ => Value::Object(std::ptr::read(ptr as *const AbstractObject)),
        },
    }
}

// TODO: unify with above
unsafe fn read_value2<'a>(ptr: *const u8, field_type: &ArrayEntryType) -> Value<'a> {
    match field_type {
        ArrayEntryType::Base(BaseType::Boolean)
        | ArrayEntryType::Base(BaseType::Byte)
        | ArrayEntryType::Base(BaseType::Char)
        | ArrayEntryType::Base(BaseType::Short)
        | ArrayEntryType::Base(BaseType::Int) => Value::Int(std::ptr::read(ptr as *const i32)),
        ArrayEntryType::Base(BaseType::Long) => Value::Long(std::ptr::read(ptr as *const i64)),
        ArrayEntryType::Base(BaseType::Float) => Value::Float(std::ptr::read(ptr as *const f32)),
        ArrayEntryType::Base(BaseType::Double) => Value::Double(std::ptr::read(ptr as *const f64)),
        ArrayEntryType::Object(_) | ArrayEntryType::Array => {
            match std::ptr::read(ptr as *const i64) {
                0 => Value::Null,
                _ => Value::Object(std::ptr::read(ptr as *const AbstractObject)),
            }
        }
    }
}

// As objects

impl<'a> AbstractObject<'a> {
    fn object_header(&self) -> &ObjectHeader {
        unsafe {
            let ptr = self.data.add(ALLOC_HEADER_SIZE);
            let header_ptr = ptr as *const ObjectHeader;
            &*header_ptr
        }
    }

    unsafe fn field_ptr(&self, field_index: usize) -> *mut u8 {
        let preceding_fields_size = 8 * field_index;
        let offset = ALLOC_HEADER_SIZE + OBJECT_HEADER_SIZE + preceding_fields_size;
        self.data.add(offset)
    }
}

impl<'a> Object<'a> for AbstractObject<'a> {
    fn class_id(&self) -> ClassId {
        self.object_header().class_id
    }

    fn set_field(&self, index: usize, value: Value<'a>) {
        unsafe {
            let ptr = self.field_ptr(index);
            write_value(ptr, value);
        }
    }

    fn get_field(&self, object_class: ClassRef, index: usize) -> Value<'a> {
        let field = object_class.field_at_index(index).unwrap();
        unsafe {
            let ptr = self.field_ptr(index);
            read_value(ptr, &field.type_descriptor)
        }
    }

    fn as_abstract_object(&self) -> AbstractObject {
        AbstractObject {
            data: self.data,
            marker: PhantomData,
        }
    }
}

// As arrays

impl<'a> AbstractObject<'a> {
    fn array_header(&self) -> &ArrayHeader {
        unsafe {
            let ptr = self.data.add(ALLOC_HEADER_SIZE);
            let header_ptr = ptr as *const ArrayHeader;
            &*header_ptr
        }
    }

    unsafe fn entry_ptr(&self, field_index: usize) -> *mut u8 {
        let entry_location = 8 * field_index;
        let offset = ALLOC_HEADER_SIZE + ARRAY_HEADER_SIZE + entry_location;
        self.data.add(offset)
    }
}

impl<'a> Array<'a> for AbstractObject<'a> {
    fn elements_type(&self) -> ArrayEntryType {
        self.array_header().elements_type.clone()
    }

    fn len(&self) -> u32 {
        self.array_header().length
    }

    fn set_element(&self, index: usize, value: Value<'a>) -> Result<(), VmError> {
        if index >= self.len().into_usize_safe() {
            Err(VmError::ArrayIndexOutOfBoundsException)
        } else {
            unsafe {
                let ptr = self.entry_ptr(index);
                write_value(ptr, value);
            }
            Ok(())
        }
    }

    fn get_element(&self, index: usize) -> Result<Value<'a>, VmError> {
        if index >= self.len().into_usize_safe() {
            Err(VmError::ArrayIndexOutOfBoundsException)
        } else {
            unsafe {
                let ptr = self.entry_ptr(index);
                Ok(read_value2(ptr, &self.elements_type()))
            }
        }
    }

    fn as_abstract_object(&self) -> AbstractObject {
        AbstractObject {
            data: self.data,
            marker: PhantomData,
        }
    }
}

pub fn string_from_char_array(array: AbstractObject) -> Result<String, VmError> {
    if array.kind() != ObjectKind::Array {
        return Err(VmError::ValidationException);
    }

    if array.elements_type() != ArrayEntryType::Base(BaseType::Char) {
        return Err(VmError::ValidationException);
    }

    let len = array.len().into_usize_safe();
    let mut string_chars: Vec<u16> = Vec::with_capacity(len);
    unsafe {
        let ptr = array.data.add(ALLOC_HEADER_SIZE + ARRAY_HEADER_SIZE) as *const i64;
        for i in 0..len {
            let ptr = ptr.add(i);
            let next_codepoint = std::ptr::read(ptr as *const i32) as u16;
            string_chars.push(next_codepoint);
        }
    }

    let string = String::from_utf16(&string_chars).expect("should have valid utf8 bytes");
    Ok(string)
}
