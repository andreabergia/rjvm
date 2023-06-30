use std::{fmt, fmt::Formatter};

use rjvm_reader::{
    class_access_flags::ClassAccessFlags, class_file_field::ClassFileField,
    class_file_method::ClassFileMethod, constant_pool::ConstantPool,
};

/// In various data structures, we store the class id of the object, i..e. a progressive
/// number assigned when we load the class. Note that, while we do not support it yet,
/// multiple class loaders could load the same class more than once, but they would be
/// required to assign different id to them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ClassId(u32);

impl fmt::Display for ClassId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ClassId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

/// A loaded java class
#[derive(Debug)]
pub struct Class<'a> {
    pub id: ClassId,
    pub name: String,
    /// Source file is stored as an attribute in the .class file, but might be missing
    /// for synthetic classes or if the compiler didn't write it.
    pub source_file: Option<String>,
    pub constants: ConstantPool,
    pub flags: ClassAccessFlags,
    pub superclass: Option<ClassRef<'a>>,
    pub interfaces: Vec<ClassRef<'a>>,
    pub fields: Vec<ClassFileField>,
    pub methods: Vec<ClassFileMethod>,
    // Base classes field have the same index they have in the base class, and our own
    // field come after. This is the index of the first "owned" field.
    // Note that this will include the static fields, as required by the bytecode specs.
    pub first_field_index: usize,
    // The total number of fields in this class, including those in the base class.
    pub num_total_fields: usize,
}

pub type ClassRef<'a> = &'a Class<'a>;

impl<'a> Class<'a> {
    /// Returns whether self is a subclass of the given class, or implements
    /// the given interface
    pub fn is_subclass_of(&self, base: ClassRef) -> bool {
        self.name == base.name
            || self
                .superclass
                .map_or(false, |superclass| superclass.is_subclass_of(base))
            || self.interfaces.iter().any(|intf| intf.is_subclass_of(base))
    }

    pub fn find_method(
        &self,
        method_name: &str,
        type_descriptor: &str,
    ) -> Option<&ClassFileMethod> {
        // Maybe replace linear search with something faster...
        self.methods
            .iter()
            .find(|method| method.name == method_name && method.type_descriptor == type_descriptor)
    }

    pub fn find_field(&self, field_name: &str) -> Option<(usize, &ClassFileField)> {
        // Maybe replace linear search with something faster...
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

    pub fn all_fields(&self) -> impl Iterator<Item = &ClassFileField> {
        let mut all_fields = Vec::from_iter(
            self.superclass
                .iter()
                .flat_map(|superclass| superclass.all_fields()),
        );
        all_fields.extend(self.fields.iter());
        all_fields.into_iter()
    }
}
