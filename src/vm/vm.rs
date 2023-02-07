use crate::reader::class_file::ClassFile;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Vm {
    classes: HashMap<String, ClassFile>,
}

impl Vm {
    pub fn load_class(&mut self, class_file: ClassFile) {
        self.classes.insert(class_file.name.clone(), class_file);
    }

    pub fn find_class(&self, class_name: &str) -> Option<&ClassFile> {
        self.classes.get(class_name)
    }
}

impl Vm {
    pub fn new() -> Vm {
        Default::default()
    }
}
