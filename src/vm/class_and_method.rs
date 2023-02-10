use crate::reader::class_file::ClassFile;
use crate::reader::class_file_method::ClassFileMethod;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct ClassAndMethod {
    pub class: Rc<ClassFile>,
    pub method: Rc<ClassFileMethod>,
}

impl ClassAndMethod {
    pub fn num_arguments(&self) -> usize {
        self.method.parsed_type_descriptor.num_arguments()
    }
}
