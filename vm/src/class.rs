use std::ops::Deref;
use std::ptr::NonNull;
use std::rc::Rc;

use rjvm_reader::{
    class_access_flags::ClassAccessFlags, class_file_field::ClassFileField,
    class_file_method::ClassFileMethod, constant_pool::ConstantPool,
};

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub constants: ConstantPool,
    pub flags: ClassAccessFlags,
    pub superclass: Option<ClassPtr>,
    pub interfaces: Vec<ClassPtr>,
    pub fields: Vec<ClassFileField>,
    pub methods: Vec<Rc<ClassFileMethod>>,
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct ClassPtr {
    inner: NonNull<Class>,
}

impl ClassPtr {
    pub fn new(class: &mut Class) -> Self {
        let inner = NonNull::new(class).unwrap();
        Self { inner }
    }
}

impl Deref for ClassPtr {
    type Target = Class;

    fn deref(&self) -> &Self::Target {
        // SAFETY: The pointer is of type NonNull.
        // Classes should never be moved because they should only be allocated by ClassAllocator,
        // that by design will never move objects.
        unsafe { self.inner.as_ref() }
    }
}

impl Class {
    pub fn find_method(
        &self,
        method_name: &str,
        type_descriptor: &str,
    ) -> Option<Rc<ClassFileMethod>> {
        // TODO: replace linear search with something faster
        self.methods
            .iter()
            .find(|method| method.name == method_name && method.type_descriptor == type_descriptor)
            .cloned()
    }

    pub fn find_field(&self, field_name: &str) -> Option<(usize, &ClassFileField)> {
        // TODO: replace linear search with something faster
        self.fields
            .iter()
            .enumerate()
            .find(|entry| entry.1.name == field_name)
    }
}
