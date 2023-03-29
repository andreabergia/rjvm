use log::info;

use crate::{class_file::ClassFile, class_reader};

pub fn read_class_from_bytes(bytes: &[u8]) -> ClassFile {
    let class = class_reader::read_buffer(bytes).unwrap();
    info!("read class file: {}", class);
    class
}
