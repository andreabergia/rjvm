use std::collections::HashMap;

use crate::class::ClassId;
use crate::{class::ClassRef, class_allocator::ClassResolver};

// TODO: class loaders should be a hierarchy

#[derive(Debug, Default)]
pub struct ClassLoader<'a> {
    classes_by_id: HashMap<ClassId, ClassRef<'a>>,
    classes_by_name: HashMap<String, ClassRef<'a>>,
}

impl<'a> ClassResolver<'a> for ClassLoader<'a> {
    fn find_class_by_id(&self, id: ClassId) -> Option<ClassRef<'a>> {
        self.classes_by_id.get(&id).cloned()
    }

    fn find_class_by_name<'b>(&'b self, name: &str) -> Option<ClassRef<'a>> {
        self.classes_by_name.get(name).cloned()
    }
}

impl<'a> ClassLoader<'a> {
    pub fn register_class(&mut self, class: ClassRef<'a>) {
        self.classes_by_id.insert(class.id, class);
        self.classes_by_name.insert(class.name.clone(), class);
    }
}
