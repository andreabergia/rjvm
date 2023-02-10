use std::path::PathBuf;

use log::info;
use rjvm::reader::class_file::ClassFile;
use rjvm::reader::class_reader;

pub fn read_class_from_file(file: &str) -> ClassFile {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/resources");
    path.push(String::from(file) + ".class");
    info!("attempting to read class from file: {}", path.display());

    let class = class_reader::read(path.as_path()).unwrap();
    info!("read class file: {}", class);
    class
}
