extern crate rjvm;

use std::path::PathBuf;

use rjvm::class_file_field::ClassFileField;
use rjvm::field_flags::FieldFlags;
use rjvm::{
    class_access_flags::ClassAccessFlags, class_file_version::ClassFileVersion, class_reader,
};

#[test]
fn can_read_class_file() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/resources/rjvm/Complex.class");

    let class = class_reader::read(path.as_path()).unwrap();
    println!("Read class file: {}", class);
    assert_eq!(ClassFileVersion::Jdk6, class.version);
    assert_eq!(
        ClassAccessFlags::PUBLIC | ClassAccessFlags::SUPER,
        class.flags
    );
    assert_eq!("rjvm/Complex", class.name);
    assert_eq!("java/lang/Object", class.superclass);
    assert_eq!(
        vec!("java/lang/Cloneable", "java/io/Serializable"),
        class.interfaces
    );
    assert_eq!(
        vec!(
            ClassFileField {
                flags: FieldFlags::PRIVATE | FieldFlags::FINAL,
                name: "real".to_string(),
                type_descriptor: "D".to_string(),
                attributes: vec![],
            },
            ClassFileField {
                flags: FieldFlags::PRIVATE | FieldFlags::FINAL,
                name: "imag".to_string(),
                type_descriptor: "D".to_string(),
                attributes: vec![],
            }
        ),
        class.fields
    );
}
