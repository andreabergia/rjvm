use std::fmt::Debug;

use rjvm_reader::field_type::{BaseType, FieldType};

use crate::abstract_object::{AbstractObject, Array2, Object2, ObjectKind};
use crate::{class::ClassRef, class_resolver_by_id::ClassByIdResolver, vm_error::VmError};

#[derive(Debug, Default, Clone, PartialEq)]
pub enum Value<'a> {
    #[default]
    Uninitialized,
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Object(AbstractObject<'a>),
    Null,
    // TODO: return address
}

impl<'a> Value<'a> {
    pub fn matches_type<'b, 'c, ResByName>(
        &self,
        expected_type: FieldType,
        class_resolver_by_id: &impl ClassByIdResolver<'c>,
        class_resolver_by_name: ResByName,
    ) -> bool
    where
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

            Value::Object(object) => {
                if object.kind() == ObjectKind::Array {
                    match expected_type {
                        FieldType::Array(expected_field_type) => {
                            let array_entry_type =
                                object.elements_type().into_field_type(class_resolver_by_id);
                            if let Some(array_entry_type) = array_entry_type {
                                array_entry_type == *expected_field_type
                            } else {
                                false
                            }
                        }
                        _ => false,
                    }
                } else {
                    match expected_type {
                        // TODO: with multiple class loaders, we should check the class identity,
                        //  not the name, since the same class could be loaded by multiple class loader
                        FieldType::Object(expected_class_name) => {
                            let value_class =
                                class_resolver_by_id.find_class_by_id(object.class_id());
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
                    }
                }
            }

            Value::Null => match expected_type {
                FieldType::Base(_) => false,
                FieldType::Object(_) => true,
                FieldType::Array(_) => true,
            },
        }
    }
}

pub fn expect_abstract_object_at<'a>(
    vec: &[Value<'a>],
    index: usize,
) -> Result<AbstractObject<'a>, VmError> {
    let value = vec.get(index);
    if let Some(Value::Object(object)) = value {
        Ok(object.clone())
    } else {
        Err(VmError::ValidationException)
    }
}

pub fn expect_concrete_object_at<'a>(
    vec: &[Value<'a>],
    index: usize,
) -> Result<impl Object2<'a>, VmError> {
    let value = expect_abstract_object_at(vec, index)?;
    if value.kind() == ObjectKind::Object {
        Ok(value)
    } else {
        Err(VmError::ValidationException)
    }
}

pub fn expect_array_at<'a>(vec: &[Value<'a>], index: usize) -> Result<impl Array2<'a>, VmError> {
    let value = expect_abstract_object_at(vec, index)?;
    if value.kind() == ObjectKind::Array {
        Ok(value)
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

pub fn expect_receiver(receiver: Option<AbstractObject>) -> Result<AbstractObject, VmError> {
    match receiver {
        Some(v) => Ok(v),
        None => Err(VmError::ValidationException),
    }
}
