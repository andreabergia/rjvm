use rjvm_reader::{class_file_method::ClassFileMethod, field_type::FieldType};

use crate::class::ClassRef;

/// A pair of a class and a method, used to avoid passing around two arguments
#[derive(Debug, Clone)]
pub struct ClassAndMethod<'a> {
    pub class: ClassRef<'a>,
    pub method: &'a ClassFileMethod,
}

impl<'a> ClassAndMethod<'a> {
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
