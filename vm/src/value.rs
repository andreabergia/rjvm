use std::{
    cell::RefCell,
    fmt::{Debug, Formatter},
    rc::Rc,
};

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
    Object(ObjectRef<'a>),
    Null, // TODO: should this be merged with Object and use an Option?

    // TODO: avoid RC and use garbage collector to allocate
    Array(FieldType, ArrayRef<'a>),
    // TODO: return address
}

#[derive(Clone, PartialEq)]
pub struct ObjectValue<'a> {
    pub class_id: ClassId,
    fields: RefCell<Vec<Value<'a>>>,
}

impl<'a> ObjectValue<'a> {
    pub fn new(class: &Class<'a>) -> Self {
        let fields = (0..class.num_total_fields)
            .map(|index| {
                let field = class.field_at_index(index).unwrap();
                match &field.type_descriptor {
                    FieldType::Base(base_type) => match base_type {
                        BaseType::Byte => Value::Int(0),
                        BaseType::Char => Value::Int(0),
                        BaseType::Double => Value::Double(0f64),
                        BaseType::Float => Value::Float(0f32),
                        BaseType::Int => Value::Int(0),
                        BaseType::Long => Value::Long(0),
                        BaseType::Short => Value::Int(0),
                        BaseType::Boolean => Value::Int(0),
                    },
                    FieldType::Object(_) => Value::Null,
                    FieldType::Array(_) => Value::Null,
                }
            })
            .collect();
        Self {
            class_id: class.id,
            fields: RefCell::new(fields),
        }
    }

    pub fn set_field(&self, index: usize, value: Value<'a>) {
        self.fields.borrow_mut()[index] = value;
    }

    pub fn get_field(&self, index: usize) -> Value<'a> {
        self.fields.borrow()[index].clone()
    }
}

pub type ObjectRef<'a> = &'a ObjectValue<'a>;
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
                    let value_class = class_resolver_by_id(object_ref.class_id);
                    if let Some(object_class) = value_class {
                        let expected_class = class_resolver_by_name(&expected_class_name);
                        expected_class.map_or(false, |expected_class| {
                            object_class.is_instance_of(expected_class)
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
        write!(f, "class: {} fields {:?}", self.class_id, self.fields)
    }
}

pub fn expect_object_at<'a>(vec: &[Value<'a>], index: usize) -> Result<ObjectRef<'a>, VmError> {
    let value = vec.get(index);
    if let Some(Value::Object(object)) = value {
        Ok(object)
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
