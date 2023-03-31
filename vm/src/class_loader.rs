use std::collections::HashMap;

use crate::class::ClassRef;

// TODO: class loaders should be a hierarchy

#[derive(Debug, Default)]
pub struct ClassLoader<'a> {
    classes_by_name: HashMap<String, ClassRef<'a>>,
}

// TODO: we should use this!
#[allow(dead_code)]
impl<'a> ClassLoader<'a> {
    pub fn register_class(&mut self, class: ClassRef<'a>) {
        self.classes_by_name.insert(class.name.clone(), class);
    }

    pub fn find_class_by_name(&self, name: &str) -> Option<ClassRef<'a>> {
        self.classes_by_name.get(name).cloned()
    }
}
