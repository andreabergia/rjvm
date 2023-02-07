use crate::reader::class_file::ClassFile;
use crate::reader::class_file_method::ClassFileMethod;

#[derive(Debug)]
pub struct ClassAndMethod<'a> {
    pub class: &'a ClassFile,
    pub method: &'a ClassFileMethod,
}
