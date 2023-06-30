extern crate rjvm_reader;

use rjvm_reader::{
    class_file_field::{ClassFileField, FieldConstantValue},
    field_flags::FieldFlags,
    field_type::{BaseType, FieldType},
};
use utils::read_class_from_bytes;

use crate::utils;

#[test_log::test]
fn can_read_constants() {
    let class = read_class_from_bytes(include_bytes!("../resources/rjvm/Constants.class"));
    assert_eq!(
        vec!(
            ClassFileField {
                flags: FieldFlags::PUBLIC | FieldFlags::STATIC | FieldFlags::FINAL,
                name: "AN_INT".to_string(),
                type_descriptor: FieldType::Base(BaseType::Int),
                constant_value: Some(FieldConstantValue::Int(2023)),
                deprecated: false,
            },
            ClassFileField {
                flags: FieldFlags::PROTECTED | FieldFlags::STATIC | FieldFlags::FINAL,
                name: "A_FLOAT".to_string(),
                type_descriptor: FieldType::Base(BaseType::Float),
                constant_value: Some(FieldConstantValue::Float(20.23)),
                deprecated: false,
            },
            ClassFileField {
                flags: FieldFlags::PRIVATE | FieldFlags::STATIC | FieldFlags::FINAL,
                name: "A_LONG".to_string(),
                type_descriptor: FieldType::Base(BaseType::Long),
                constant_value: Some(FieldConstantValue::Long(2023)),
                deprecated: false,
            },
            ClassFileField {
                flags: FieldFlags::PUBLIC | FieldFlags::STATIC | FieldFlags::FINAL,
                name: "A_DOUBLE".to_string(),
                type_descriptor: FieldType::Base(BaseType::Double),
                constant_value: Some(FieldConstantValue::Double(20.23)),
                deprecated: false,
            },
            ClassFileField {
                flags: FieldFlags::PUBLIC | FieldFlags::STATIC | FieldFlags::FINAL,
                name: "A_STRING".to_string(),
                type_descriptor: FieldType::Object("java/lang/String".to_string()),
                constant_value: Some(FieldConstantValue::String("2023".to_string())),
                deprecated: false,
            }
        ),
        class.fields
    );
}
