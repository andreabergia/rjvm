use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use typed_arena::Arena;

use crate::class::{Class, ClassPtr, ClassResolver};

pub struct ClassArena {
    arena: Arena<Class>,
}

impl Default for ClassArena {
    fn default() -> Self {
        Self {
            arena: Arena::with_capacity(100),
        }
    }
}

impl fmt::Debug for ClassArena {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "class_arena={{len={}}}", self.arena.len())
    }
}

impl ClassArena {
    pub fn allocate(&mut self, class: Class) -> ClassPtr {
        ClassPtr::new(self.arena.alloc(class))
    }
}

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
