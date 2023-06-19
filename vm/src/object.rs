use std::{
    fmt::{Debug, Formatter},
    marker::PhantomData,
};

use rjvm_reader::field_type::{BaseType, FieldType};

use crate::{
    array::Array,
    class::{Class, ClassId, ClassRef},
    value::Value,
};

#[derive(PartialEq, Clone)]
#[repr(transparent)]
pub struct Object<'a> {
    data: *mut u8,
    marker: PhantomData<&'a [u8]>,
}

const HEADER_SIZE: usize = 8;

impl<'a> Object<'a> {
    pub(crate) fn size(class: &Class<'a>) -> usize {
        /*
        let fields_sizes: usize = (0..class.num_total_fields)
                    .map(|index| class.field_at_index(index).unwrap())
                    .map(|field| field_size(field))
                    .sum();

        fn field_size(field: &ClassFileField) -> usize {
            match &field.type_descriptor {
                FieldType::Base(base_type) => match base_type {
                    BaseType::Byte
                    | BaseType::Char
                    | BaseType::Float
                    | BaseType::Int
                    | BaseType::Short
                    | BaseType::Boolean => 4,
                    BaseType::Double | BaseType::Long => 8,
                },
                FieldType::Object(_) => std::mem::size_of::<*mut ObjectValue<'a>>(),
                FieldType::Array(_) => std::mem::size_of::<*mut ObjectValue<'a>>(),
            }
        }
        */

        let fields_sizes: usize = 8 * class.num_total_fields;
        fields_sizes + HEADER_SIZE
    }

    pub fn new(class: &Class<'a>, ptr: *mut u8) -> Self {
        // Header is composed of
        //   32 bits for the class id
        //   32 bits for the identity hash code
        let header = (class.id.as_u32() as u64) << 32 | identity_hash_code(ptr) as u64;
        unsafe {
            let header_ptr = ptr as *mut u64;
            std::ptr::write(header_ptr, header);
        };

        Self {
            data: ptr,
            marker: PhantomData::default(),
        }
    }

    pub fn class_id(&self) -> ClassId {
        ClassId::new((self.header() >> 32) as u32)
    }

    pub fn identity_hash_code(&self) -> i32 {
        (self.header() & 0xFFFFFFFF) as i32
    }

    fn header(&self) -> u64 {
        unsafe {
            let header_ptr = self.data as *mut u64;
            std::ptr::read(header_ptr)
        }
    }

    pub fn set_field(&self, index: usize, value: Value<'a>) {
        let preceding_fields_size = 8 * index;
        let offset = HEADER_SIZE + preceding_fields_size;
        unsafe {
            let ptr = self.data.add(offset);
            match value {
                Value::Int(int) => std::ptr::write(ptr as *mut i32, int),
                Value::Long(long) => std::ptr::write(ptr as *mut i64, long),
                Value::Float(float) => std::ptr::write(ptr as *mut f32, float),
                Value::Double(double) => std::ptr::write(ptr as *mut f64, double),
                Value::Uninitialized | Value::Null => std::ptr::write(ptr as *mut u64, 0),
                Value::Object(obj) => std::ptr::write(ptr as *mut Object, obj),
                Value::Array(arr) => std::ptr::write(ptr as *mut Array, arr),
            }
        }
    }

    pub fn get_field(&self, object_class: ClassRef, index: usize) -> Value<'a> {
        let field = object_class.field_at_index(index).unwrap();

        let preceding_fields_size = 8 * index;
        let offset = HEADER_SIZE + preceding_fields_size;
        unsafe {
            let ptr = self.data.add(offset);
            match &field.type_descriptor {
                FieldType::Base(BaseType::Boolean)
                | FieldType::Base(BaseType::Byte)
                | FieldType::Base(BaseType::Char)
                | FieldType::Base(BaseType::Short)
                | FieldType::Base(BaseType::Int) => Value::Int(std::ptr::read(ptr as *const i32)),
                FieldType::Base(BaseType::Long) => Value::Long(std::ptr::read(ptr as *const i64)),
                FieldType::Base(BaseType::Float) => Value::Float(std::ptr::read(ptr as *const f32)),
                FieldType::Base(BaseType::Double) => {
                    Value::Double(std::ptr::read(ptr as *const f64))
                }
                FieldType::Object(_) => match std::ptr::read(ptr as *const i64) {
                    0 => Value::Null,
                    _ => Value::Object(std::ptr::read(ptr as *const Object)),
                },
                FieldType::Array(_) => match std::ptr::read(ptr as *const i64) {
                    0 => Value::Null,
                    _ => Value::Array(std::ptr::read(ptr as *const Array)),
                },
            }
        }
    }

    pub(crate) unsafe fn offset_of_field(&self, index: usize) -> *mut u8 {
        let preceding_fields_size = 8 * index;
        let offset = HEADER_SIZE + preceding_fields_size;
        self.data.add(offset)
    }

    // TODO: impl eq
    pub fn is_same_as(&self, other: &Object) -> bool {
        self.data == other.data
    }
}

fn identity_hash_code(ptr: *mut u8) -> u32 {
    let addr = ptr as u64;
    let hash = (addr >> 32) ^ (addr);
    hash as u32
}

impl<'a> Debug for Object<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "class:{}, data:{:#0x}",
            self.class_id(),
            self.data as usize
        )
    }
}
