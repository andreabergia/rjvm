use std::collections::HashMap;

use crate::class::ClassPtr;
use crate::class_allocator::ClassResolver;

// TODO: class loaders should be a hierarchy

#[derive(Debug, Default)]
pub struct ClassLoader {
    classes: HashMap<String, ClassPtr>,
}

impl ClassResolver for ClassLoader {
    fn find_class(&self, name: &str) -> Option<ClassPtr> {
        self.classes.get(name).cloned()
    }
}

impl ClassLoader {
    pub fn register_class(&mut self, class: ClassPtr) {
        self.classes.insert(class.name.clone(), class);
    }
}
