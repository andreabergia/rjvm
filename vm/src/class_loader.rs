use std::collections::HashMap;

use crate::class::Class;
use crate::class_allocator::ClassResolver;

// TODO: class loaders should be a hierarchy

#[derive(Debug, Default)]
pub struct ClassLoader<'a> {
    classes: HashMap<String, &'a Class<'a>>,
}

impl<'a> ClassResolver<'a> for ClassLoader<'a> {
    fn find_class<'b>(&'b self, name: &str) -> Option<&'a Class<'a>> {
        self.classes.get(name).cloned()
    }
}

impl<'a> ClassLoader<'a> {
    pub fn register_class(&mut self, class: &'a Class<'a>) {
        self.classes.insert(class.name.clone(), class);
    }
}
