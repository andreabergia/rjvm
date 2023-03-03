use std::collections::HashMap;

use crate::{class::ClassRef, class_allocator::ClassResolver};

// TODO: class loaders should be a hierarchy

#[derive(Debug, Default)]
pub struct ClassLoader<'a> {
    classes: HashMap<String, ClassRef<'a>>,
}

impl<'a> ClassResolver<'a> for ClassLoader<'a> {
    fn find_class<'b>(&'b self, name: &str) -> Option<ClassRef<'a>> {
        self.classes.get(name).cloned()
    }
}

impl<'a> ClassLoader<'a> {
    pub fn register_class(&mut self, class: ClassRef<'a>) {
        self.classes.insert(class.name.clone(), class);
    }
}
