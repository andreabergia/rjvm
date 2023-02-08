use crate::reader::class_file::ClassFile;
use crate::reader::class_file_method::ClassFileMethod;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct ClassAndMethod {
    pub class: Rc<ClassFile>,
    pub method: Rc<ClassFileMethod>,
}
