use std::path::PathBuf;

use rjvm::{
    class_file::{ClassAccessFlags, JAVA6_CLASSFILE},
    class_reader,
};

extern crate rjvm;

#[test]
fn can_read_class_file() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/resources/rjvm/Complex.class");

    let class = class_reader::read(path.as_path()).unwrap();
    assert_eq!(JAVA6_CLASSFILE, class.major_version);
    assert_eq!(0, class.minor_version);
    assert_eq!(
        ClassAccessFlags::PUBLIC | ClassAccessFlags::SUPER,
        class.flags
    );
    assert_eq!("rjvm/Complex", class.name);
    assert_eq!("java/lang/Object", class.superclass);
}
