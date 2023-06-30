extern crate rjvm_reader;

use rjvm_reader::{
    class_access_flags::ClassAccessFlags,
    class_file::ClassFile,
    class_file_field::ClassFileField,
    class_file_version::ClassFileVersion,
    field_flags::FieldFlags,
    field_type::{BaseType, FieldType},
    line_number::LineNumber,
    line_number_table::{LineNumberTable, LineNumberTableEntry},
    method_flags::MethodFlags,
    program_counter::ProgramCounter,
};
use utils::read_class_from_bytes;

use crate::{assertions::check_method, utils};

#[test_log::test]
fn can_read_pojo_class_file() {
    let class = read_class_from_bytes(include_bytes!("../resources/rjvm/Complex.class"));
    assert_eq!(ClassFileVersion::Jdk6, class.version);
    assert_eq!(
        ClassAccessFlags::PUBLIC | ClassAccessFlags::SUPER,
        class.flags
    );
    assert_eq!("rjvm/Complex", class.name);
    assert_eq!(
        "java/lang/Object",
        class.superclass.as_ref().expect("a valid superclass")
    );
    assert_eq!(
        vec!("java/lang/Cloneable", "java/io/Serializable"),
        class.interfaces
    );
    assert_eq!(Some("Complex.java".to_string()), class.source_file);

    check_fields(&class);
    check_methods(&class);
}

fn check_fields(class: &ClassFile) {
    assert_eq!(
        vec!(
            ClassFileField {
                flags: FieldFlags::PRIVATE | FieldFlags::FINAL,
                name: "real".to_string(),
                type_descriptor: FieldType::Base(BaseType::Double),
                constant_value: None,
                deprecated: false,
            },
            ClassFileField {
                flags: FieldFlags::PRIVATE | FieldFlags::FINAL,
                name: "imag".to_string(),
                type_descriptor: FieldType::Base(BaseType::Double),
                constant_value: None,
                deprecated: false,
            }
        ),
        class.fields
    );
}

fn check_methods(class: &ClassFile) {
    assert_eq!(5, class.methods.len());

    check_method(&class.methods[0], MethodFlags::PUBLIC, "<init>", "(D)V");
    assert_eq!(
        Some(LineNumberTable::new(vec![
            LineNumberTableEntry::new(ProgramCounter(0), LineNumber(9)),
            LineNumberTableEntry::new(ProgramCounter(4), LineNumber(10)),
            LineNumberTableEntry::new(ProgramCounter(9), LineNumber(11)),
            LineNumberTableEntry::new(ProgramCounter(14), LineNumber(12)),
        ])),
        class.methods[0].code.as_ref().unwrap().line_number_table
    );

    check_method(&class.methods[1], MethodFlags::PUBLIC, "<init>", "(DD)V");
    check_method(&class.methods[2], MethodFlags::PUBLIC, "getReal", "()D");
    check_method(&class.methods[3], MethodFlags::PUBLIC, "getImag", "()D");
    check_method(&class.methods[4], MethodFlags::PUBLIC, "abs", "()D");
    assert_eq!(
        Some(LineNumberTable::new(vec![LineNumberTableEntry::new(
            ProgramCounter(0),
            LineNumber(28)
        )])),
        class.methods[4].code.as_ref().unwrap().line_number_table
    );
}
