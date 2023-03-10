use std::fmt;
use std::fmt::Formatter;

use rjvm_reader::{
    class_access_flags::ClassAccessFlags, class_file_field::ClassFileField,
    class_file_method::ClassFileMethod, constant_pool::ConstantPool,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClassId(u64);

impl fmt::Display for ClassId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ClassId {
    pub(crate) fn new(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Debug)]
pub struct Class<'a> {
    pub id: ClassId,
    pub name: String,
    pub constants: ConstantPool,
    pub flags: ClassAccessFlags,
    pub superclass: Option<ClassRef<'a>>,
    pub interfaces: Vec<ClassRef<'a>>,
    pub fields: Vec<ClassFileField>,
    pub methods: Vec<ClassFileMethod>,
    pub first_field_index: usize,
    pub num_total_fields: usize,
}

pub type ClassRef<'a> = &'a Class<'a>;

impl<'a> Class<'a> {
    pub fn find_method(
        &self,
        method_name: &str,
        type_descriptor: &str,
    ) -> Option<&ClassFileMethod> {
        // TODO: replace linear search with something faster
        self.methods
            .iter()
            .find(|method| method.name == method_name && method.type_descriptor == type_descriptor)
    }

    pub fn find_field(
        &self,
        class_name: &str,
        field_name: &str,
    ) -> Option<(usize, &ClassFileField)> {
        // TODO: maybe replace linear search with something faster?
        if class_name == self.name {
            self.fields
                .iter()
                .enumerate()
                .find(|entry| entry.1.name == field_name)
                .map(|(index, field)| (index + self.first_field_index, field))
        } else if let Some(superclass) = &self.superclass {
            superclass.find_field(class_name, field_name)
        } else {
            None
        }
    }
}
