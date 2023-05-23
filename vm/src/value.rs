use std::marker::PhantomData;
use std::{
    cell::RefCell,
    fmt::{Debug, Formatter},
    rc::Rc,
};

use log::debug;

use rjvm_reader::field_type::{BaseType, FieldType};

use crate::{
    class::{Class, ClassId, ClassRef},
    vm_error::VmError,
};

#[derive(Debug, Default, Clone, PartialEq)]
pub enum Value<'a> {
    #[default]
    Uninitialized,
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Object(ObjectValue<'a>),
    Null, // TODO: should this be merged with Object and use an Option?

    // TODO: avoid RC and use garbage collector to allocate
    Array(FieldType, ArrayRef<'a>),
    // TODO: return address
}

#[derive(PartialEq, Clone)]
pub struct ObjectValue<'a> {
    data: *mut u8,
    marker: PhantomData<Value<'a>>,
}

const HEADER_SIZE: usize = 8;

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

impl<'a> ObjectValue<'a> {
    pub(crate) fn size(class: &Class<'a>) -> usize {
        let fields_sizes: usize = 8 * class.num_total_fields;
        let object_size = fields_sizes + HEADER_SIZE;
        debug!(
            "object of class {} should have size {}",
            class.name, object_size
        );
        object_size
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

    pub fn get_class_id(&self) -> ClassId {
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
        let preceding_fields_size: usize = 8 * index;
        let offset = HEADER_SIZE + preceding_fields_size;
        unsafe {
            let ptr = self.data.add(offset);
            match value {
                Value::Int(int) => std::ptr::write(ptr as *mut i32, int),
                Value::Long(long) => std::ptr::write(ptr as *mut i64, long),
                Value::Float(float) => std::ptr::write(ptr as *mut f32, float),
                Value::Double(double) => std::ptr::write(ptr as *mut f64, double),
                Value::Uninitialized | Value::Null => std::ptr::write(ptr, 0),
                Value::Object(obj) => std::ptr::write(ptr as *mut ObjectValue, obj),
                Value::Array(_, arr) => std::ptr::write(ptr as *mut ArrayRef, arr),
            }
        }
    }

    pub fn get_field(&self, object_class: ClassRef, index: usize) -> Value<'a> {
        let field = object_class.field_at_index(index).unwrap();

        let preceding_fields_size: usize = 8 * index;
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
                FieldType::Object(_) => Value::Object(std::ptr::read(ptr as *const ObjectValue)),
                FieldType::Array(entry_type) => Value::Array(
                    entry_type.as_ref().clone(),
                    std::ptr::read(ptr as *const ArrayRef),
                ),
            }
        }
    }

    pub fn is_same_as(&self, other: &ObjectValue) -> bool {
        self.data == other.data
    }
}

fn identity_hash_code(ptr: *mut u8) -> u32 {
    let addr = ptr as u64;
    let hash = (addr >> 32) ^ (addr);
    hash as u32
}

pub type ArrayRef<'a> = Rc<RefCell<Vec<Value<'a>>>>;

impl<'a> Value<'a> {
    pub fn matches_type<'b, ResById, ResByName>(
        &self,
        expected_type: FieldType,
        class_resolver_by_id: ResById,
        class_resolver_by_name: ResByName,
    ) -> bool
    where
        ResById: FnOnce(ClassId) -> Option<ClassRef<'b>>,
        ResByName: FnOnce(&str) -> Option<ClassRef<'b>>,
    {
        match self {
            Value::Uninitialized => false,
            Value::Int(_) => match expected_type {
                FieldType::Base(base_type) => matches!(
                    base_type,
                    BaseType::Int
                        | BaseType::Byte
                        | BaseType::Char
                        | BaseType::Short
                        | BaseType::Boolean
                ),
                _ => false,
            },
            Value::Long(_) => match expected_type {
                FieldType::Base(base_type) => base_type == BaseType::Long,
                _ => false,
            },
            Value::Float(_) => match expected_type {
                FieldType::Base(base_type) => base_type == BaseType::Float,
                _ => false,
            },
            Value::Double(_) => match expected_type {
                FieldType::Base(base_type) => base_type == BaseType::Double,
                _ => false,
            },

            Value::Object(object_ref) => match expected_type {
                // TODO: with multiple class loaders, we should check the class identity,
                //  not the name, since the same class could be loaded by multiple class loader
                FieldType::Object(expected_class_name) => {
                    let value_class = class_resolver_by_id(object_ref.get_class_id());
                    if let Some(object_class) = value_class {
                        let expected_class = class_resolver_by_name(&expected_class_name);
                        expected_class.map_or(false, |expected_class| {
                            object_class.is_subclass_of(expected_class)
                        })
                    } else {
                        false
                    }
                }
                _ => false,
            },

            Value::Null => match expected_type {
                FieldType::Base(_) => false,
                FieldType::Object(_) => true,
                FieldType::Array(_) => true,
            },

            Value::Array(field_type, _) => match expected_type {
                FieldType::Array(expected_field_type) => *field_type == *expected_field_type,
                _ => false,
            },
        }
    }
}

impl<'a> Debug for ObjectValue<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "class: {} fields [", self.get_class_id())?;
        // for field in self.fields.borrow().iter() {
        //     match field {
        //         Value::Object(object) => write!(f, "object cid = {}", object.get_class_id())?,
        //         Value::Array(arr_type, arr_ref) => write!(
        //             f,
        //             "array of type {} len {}",
        //             arr_type,
        //             arr_ref.borrow().len()
        //         )?,
        //         _ => field.fmt(f)?,
        //     }
        //     write!(f, ", ")?;
        // }
        write!(f, "{:?}", self.data)?;
        write!(f, "]")
    }
}

pub fn expect_object_at<'a>(vec: &[Value<'a>], index: usize) -> Result<ObjectValue<'a>, VmError> {
    let value = vec.get(index);
    if let Some(Value::Object(object)) = value {
        Ok(object.clone())
    } else {
        Err(VmError::ValidationException)
    }
}

pub fn expect_array_at<'a, 'b>(
    vec: &'b [Value<'a>],
    index: usize,
) -> Result<(&'b FieldType, &'b ArrayRef<'a>), VmError> {
    let value = vec.get(index);
    if let Some(Value::Array(field_type, array_ref)) = value {
        Ok((field_type, array_ref))
    } else {
        Err(VmError::ValidationException)
    }
}

pub fn expect_int_at(vec: &[Value], index: usize) -> Result<i32, VmError> {
    let value = vec.get(index);
    if let Some(Value::Int(int)) = value {
        Ok(*int)
    } else {
        Err(VmError::ValidationException)
    }
}

pub fn expect_float_at(vec: &[Value], index: usize) -> Result<f32, VmError> {
    let value = vec.get(index);
    if let Some(Value::Float(float)) = value {
        Ok(*float)
    } else {
        Err(VmError::ValidationException)
    }
}

pub fn expect_double_at(vec: &[Value], index: usize) -> Result<f64, VmError> {
    let value = vec.get(index);
    if let Some(Value::Double(double)) = value {
        Ok(*double)
    } else {
        Err(VmError::ValidationException)
    }
}

pub fn expect_receiver(receiver: Option<ObjectValue>) -> Result<ObjectValue, VmError> {
    match receiver {
        Some(v) => Ok(v),
        None => Err(VmError::ValidationException),
    }
}

pub fn clone_array(array: Value) -> Result<Value, VmError> {
    match array {
        Value::Array(elements_type, array_ref) => {
            let existing_vec = array_ref.borrow();

            let mut new_vec = Vec::with_capacity(existing_vec.len());
            for value in existing_vec.iter() {
                new_vec.push(value.clone());
            }

            let new_vec = Rc::new(RefCell::new(new_vec));
            let new_array = Value::Array(elements_type, new_vec);
            Ok(new_array)
        }
        _ => Err(VmError::ValidationException),
    }
}
