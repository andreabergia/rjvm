use std::path::PathBuf;

extern crate rjvm;

#[test]
fn can_read_class_file() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/resources/rjvm/Complex.class");
    let data = std::fs::read(path).unwrap();

    let class_file = rjvm::class::read(&data).unwrap();
    assert_eq!("todo", class_file.name)
}
