use crate::class::Class;
use rjvm_reader::class_file_method::ClassFileMethod;
use rjvm_reader::field_type::FieldType;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ClassAndMethod {
    pub class: Arc<Class>,
    pub method: Rc<ClassFileMethod>,
}

impl ClassAndMethod {
    pub fn num_arguments(&self) -> usize {
        self.method.parsed_type_descriptor.num_arguments()
    }

    pub fn return_type(&self) -> Option<FieldType> {
        self.method.parsed_type_descriptor.return_type.clone()
    }

    pub fn is_static(&self) -> bool {
        self.method.is_static()
    }

    pub fn is_native(&self) -> bool {
        self.method.is_native()
    }

    pub fn is_void(&self) -> bool {
        self.method.is_void()
    }

    pub fn returns(&self, expected_type: FieldType) -> bool {
        self.method.returns(expected_type)
    }
}
