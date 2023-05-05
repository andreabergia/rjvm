use std::{fmt, fmt::Formatter};

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
    pub source_file: Option<String>,
    pub constants: ConstantPool,
    pub flags: ClassAccessFlags,
    pub superclass: Option<ClassRef<'a>>,
    pub interfaces: Vec<ClassRef<'a>>,
    pub fields: Vec<ClassFileField>,
    pub methods: Vec<ClassFileMethod>,
    pub first_field_index: usize,
    pub num_total_fields: usize,
}

impl<'a> Class<'a> {
    pub fn is_subclass_of(&self, base: ClassRef) -> bool {
        self.name == base.name
            || self
                .superclass
                .map_or(false, |superclass| superclass.is_subclass_of(base))
            || self.interfaces.iter().any(|intf| intf.is_subclass_of(base))
    }
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

    pub fn find_field(&self, field_name: &str) -> Option<(usize, &ClassFileField)> {
        // TODO: maybe replace linear search with something faster?
        self.fields
            .iter()
            .enumerate()
            .find(|entry| entry.1.name == field_name)
            .map(|(index, field)| (index + self.first_field_index, field))
            .or_else(|| {
                if let Some(superclass) = &self.superclass {
                    superclass.find_field(field_name)
                } else {
                    None
                }
            })
    }

    pub fn field_at_index(&self, index: usize) -> Option<&ClassFileField> {
        if index < self.first_field_index {
            self.superclass
                .and_then(|superclass| superclass.field_at_index(index))
        } else {
            self.fields.get(index - self.first_field_index)
        }
    }
}
