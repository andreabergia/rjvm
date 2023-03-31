use std::{collections::HashMap, fmt, fmt::Formatter};

use typed_arena::Arena;

use rjvm_reader::{class_file::ClassFile, class_reader};

use crate::{
    class::{Class, ClassId, ClassRef},
    class_loader::ClassLoader,
    class_path::{ClassPath, ClassPathParseError},
    vm_error::VmError,
};

type ClassesByName<'a> = HashMap<String, ClassRef<'a>>;

pub(crate) struct ClassManager<'a> {
    pub(crate) class_path: ClassPath,
    classes_by_id: HashMap<ClassId, ClassRef<'a>>,
    classes_by_name: ClassesByName<'a>,
    arena: Arena<Class<'a>>,
    next_id: u64,
    current_class_loader: ClassLoader<'a>,
}

impl<'a> Default for ClassManager<'a> {
    fn default() -> Self {
        Self {
            class_path: Default::default(),
            classes_by_id: Default::default(),
            classes_by_name: Default::default(),
            arena: Arena::with_capacity(100),
            next_id: 1,
            current_class_loader: Default::default(),
        }
    }
}

impl<'a> fmt::Debug for ClassManager<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "class_manager={{loaded classes={}}}", self.arena.len())
    }
}

impl<'a> ClassManager<'a> {
    pub fn append_class_path(&mut self, class_path: &str) -> Result<(), ClassPathParseError> {
        self.class_path.push(class_path)
    }

    pub fn find_class_by_id(&self, id: ClassId) -> Option<ClassRef<'a>> {
        self.classes_by_id.get(&id).cloned()
    }

    pub fn get_or_resolve_class(&mut self, class_name: &str) -> Result<ClassRef<'a>, VmError> {
        if let Some(already_loaded_class) = self.classes_by_name.get(class_name) {
            Ok(already_loaded_class)
        } else {
            self.resolve_and_load_class(class_name)
        }
    }

    fn resolve_and_load_class(&mut self, class_name: &str) -> Result<ClassRef<'a>, VmError> {
        let class_file_bytes = self
            .class_path
            .resolve(class_name)
            .map_err(|_| VmError::ClassLoadingError)?
            .ok_or(VmError::ClassNotFoundException(class_name.to_string()))?;
        let class_file =
            class_reader::read_buffer(&class_file_bytes).map_err(|_| VmError::ClassLoadingError)?;
        self.load_class(class_file)
    }

    fn load_class(&mut self, class_file: ClassFile) -> Result<ClassRef<'a>, VmError> {
        let referenced_classes = self.resolve_super_and_interfaces(&class_file)?;
        let class = self.allocate(class_file, referenced_classes)?;
        self.register_loaded_class(class);
        Ok(class)
    }

    fn resolve_super_and_interfaces(
        &mut self,
        class_file: &ClassFile,
    ) -> Result<ClassesByName<'a>, VmError> {
        let mut resolved_classes: HashMap<String, ClassRef<'a>> = Default::default();
        if let Some(superclass_name) = &class_file.superclass {
            self.resolve_and_collect_class(superclass_name, &mut resolved_classes)?;
        }
        for interface_name in class_file.interfaces.iter() {
            self.resolve_and_collect_class(interface_name, &mut resolved_classes)?;
        }
        Ok(resolved_classes)
    }

    fn resolve_and_collect_class(
        &mut self,
        class_name: &str,
        resolved_classes: &mut ClassesByName<'a>,
    ) -> Result<ClassRef<'a>, VmError> {
        let class = self.get_or_resolve_class(class_name)?;
        resolved_classes.insert(class_name.to_string(), class);
        Ok(class)
    }

    fn allocate(
        &mut self,
        class_file: ClassFile,
        referenced_classes: ClassesByName<'a>,
    ) -> Result<ClassRef<'a>, VmError> {
        let next_id = self.next_id;
        self.next_id += 1;

        let class = Self::new_class(class_file, ClassId::new(next_id), referenced_classes)?;
        let class_ref = self.arena.alloc(class);

        // SAFETY: our reference class_ref is alive only for 'b.
        // However we actually know that the arena will keep the value alive for 'a,
        // and I cannot find a way to convince the compiler of this fact. Thus
        // I'm using this pointer "trick" to make the compiler happy.
        // I'm sure this can be done with safe Rust, I just do not know how at the moment...
        unsafe {
            let class_ptr: *const Class<'a> = class_ref;
            Ok(&*class_ptr)
        }
    }

    fn new_class(
        class_file: ClassFile,
        id: ClassId,
        resolved_classes: HashMap<String, ClassRef<'a>>,
    ) -> Result<Class<'a>, VmError> {
        let superclass = class_file
            .superclass
            .as_ref()
            .map(|superclass_name| *resolved_classes.get(superclass_name).unwrap());
        let interfaces: Vec<ClassRef<'a>> = class_file
            .interfaces
            .iter()
            .map(|interface_name| *resolved_classes.get(interface_name).unwrap())
            .collect();

        let num_superclass_fields = match superclass {
            Some(superclass) => superclass.num_total_fields,
            None => 0,
        };
        let num_this_class_fields = class_file.fields.len();

        Ok(Class {
            id,
            name: class_file.name,
            constants: class_file.constants,
            flags: class_file.flags,
            superclass,
            interfaces,
            fields: class_file.fields,
            methods: class_file.methods,
            num_total_fields: num_superclass_fields + num_this_class_fields,
            first_field_index: num_superclass_fields,
        })
    }

    fn register_loaded_class(&mut self, class: ClassRef<'a>) {
        self.classes_by_name.insert(class.name.clone(), class);
        self.classes_by_id.insert(class.id, class);
        self.current_class_loader.register_class(class);
    }
}
