use std::collections::HashMap;
use std::sync::Arc;

use crate::class::{Class, ClassResolver};

// TODO: class loaders should be a hierarchy

#[derive(Debug)]
pub struct ClassLoader {
    classes: HashMap<String, Arc<Class>>,
}

impl ClassResolver for ClassLoader {
    fn find_class(&self, name: &str) -> Option<Arc<Class>> {
        self.classes.get(name).cloned()
    }
}

impl ClassLoader {
    pub fn register_class(&mut self, class: Class) {
        self.classes.insert(class.name.clone(), Arc::new(class));
    }
}
