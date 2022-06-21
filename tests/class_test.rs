use std::{io, path::PathBuf};

extern crate rjvm;

#[test]
fn magic_number_is_required() {
    let data = vec![0x00];
    let class_file = rjvm::class::read(&data).map_err(|e| e.kind());

    let expected = Err(io::ErrorKind::InvalidData);
    assert_eq!(expected, class_file);
}

#[test]
fn can_read_class_file() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/resources/Person.class");
    let data = std::fs::read(path).unwrap();

    let class_file = rjvm::class::read(&data).unwrap();
    assert_eq!("todo", class_file.name)
}
