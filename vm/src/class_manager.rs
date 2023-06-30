use std::{collections::HashMap, fmt, fmt::Formatter};

use indexmap::IndexMap;
use log::debug;
use typed_arena::Arena;

use rjvm_reader::{class_file::ClassFile, class_reader};

use crate::{
    class::{Class, ClassId, ClassRef},
    class_loader::ClassLoader,
    class_path::{ClassPath, ClassPathParseError},
    class_resolver_by_id::ClassByIdResolver,
    vm_error::VmError,
};

/// An object that will allocate and manage Class objects
pub(crate) struct ClassManager<'a> {
    class_path: ClassPath,
    classes_by_id: HashMap<ClassId, ClassRef<'a>>,
    classes_by_name: HashMap<String, ClassRef<'a>>,
    /// Used to allocate class instances that will be alive as long as the arena
    /// (and thus the `ClassManager` are alive).
    arena: Arena<Class<'a>>,

    /// Used to generate ClassId
    next_id: u32,

    /// In a real implementation, we would have a current class loader for each thread,
    /// in a hierarchy. Currently, we only have exactly ONE global class loader.
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

/// When a class instance is requested, returns whether the class was already loaded,
/// or whether the requeste loaded a new class (which will need to be initialized).
#[derive(Debug, Clone)]
pub(crate) enum ResolvedClass<'a> {
    AlreadyLoaded(ClassRef<'a>),
    NewClass(ClassesToInitialize<'a>),
}

impl<'a> ResolvedClass<'a> {
    pub fn get_class(&self) -> ClassRef<'a> {
        match self {
            ResolvedClass::AlreadyLoaded(class) => class,
            ResolvedClass::NewClass(classes_to_initialize) => classes_to_initialize.resolved_class,
        }
    }
}

/// In case a new class was loaded, maps the whole list of the classes that require
/// initialization, in order so that a base class is initialized _before_ the derived classes.
/// Includes the newly resolved class in the list [to_initialize].
#[derive(Debug, Clone)]
pub(crate) struct ClassesToInitialize<'a> {
    resolved_class: ClassRef<'a>,
    pub(crate) to_initialize: Vec<ClassRef<'a>>,
}

impl<'a> ClassByIdResolver<'a> for ClassManager<'a> {
    fn find_class_by_id(&self, id: ClassId) -> Option<ClassRef<'a>> {
        self.classes_by_id.get(&id).cloned()
    }
}

impl<'a> ClassManager<'a> {
    pub fn append_class_path(&mut self, class_path: &str) -> Result<(), ClassPathParseError> {
        self.class_path.push(class_path)
    }

    pub fn find_class_by_name(&self, class_name: &str) -> Option<ClassRef<'a>> {
        self.classes_by_name.get(class_name).cloned()
    }

    pub fn get_or_resolve_class(&mut self, class_name: &str) -> Result<ResolvedClass<'a>, VmError> {
        if let Some(already_loaded_class) = self.find_class_by_name(class_name) {
            Ok(ResolvedClass::AlreadyLoaded(already_loaded_class))
        } else {
            self.resolve_and_load_class(class_name)
                .map(ResolvedClass::NewClass)
        }
    }

    fn resolve_and_load_class(
        &mut self,
        class_name: &str,
    ) -> Result<ClassesToInitialize<'a>, VmError> {
        let class_file_bytes = self
            .class_path
            .resolve(class_name)
            .map_err(|err| VmError::ClassLoadingError(err.to_string()))?
            .ok_or(VmError::ClassNotFoundException(class_name.to_string()))?;
        let class_file = class_reader::read_buffer(&class_file_bytes)
            .map_err(|err| VmError::ClassLoadingError(err.to_string()))?;
        self.load_class(class_file)
    }

    fn load_class(&mut self, class_file: ClassFile) -> Result<ClassesToInitialize<'a>, VmError> {
        let referenced_classes = self.resolve_super_and_interfaces(&class_file)?;
        let loaded_class = self.allocate(class_file, referenced_classes)?;
        self.register_loaded_class(loaded_class.resolved_class);
        Ok(loaded_class)
    }

    fn resolve_super_and_interfaces(
        &mut self,
        class_file: &ClassFile,
    ) -> Result<IndexMap<String, ResolvedClass<'a>>, VmError> {
        let mut resolved_classes: IndexMap<String, ResolvedClass<'a>> = Default::default();
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
        resolved_classes: &mut IndexMap<String, ResolvedClass<'a>>,
    ) -> Result<(), VmError> {
        let class = self.get_or_resolve_class(class_name)?;
        resolved_classes.insert(class_name.to_string(), class);
        Ok(())
    }

    fn allocate(
        &mut self,
        class_file: ClassFile,
        referenced_classes: IndexMap<String, ResolvedClass<'a>>,
    ) -> Result<ClassesToInitialize<'a>, VmError> {
        let next_id = self.next_id;
        self.next_id += 1;

        let id = ClassId::new(next_id);
        debug!("loading class {} from file {}", id, class_file.name);
        let class = Self::new_class(class_file, id, &referenced_classes)?;
        let class_ref = self.arena.alloc(class);

        // SAFETY: our reference class_ref is alive only for 'b.
        // However we actually know that the arena will keep the value alive for 'a,
        // and I cannot find a way to convince the compiler of this fact. Thus
        // I'm using this pointer "trick" to make the compiler happy.
        // I expect this can be done with safe Rust, I just do not know how at the moment...
        let class_ref = unsafe {
            let class_ptr: *const Class<'a> = class_ref;
            &*class_ptr
        };

        let mut classes_to_init: Vec<ClassRef<'a>> = Vec::new();
        for resolved_class in referenced_classes.values() {
            if let ResolvedClass::NewClass(new_class) = resolved_class {
                for to_initialize in new_class.to_initialize.iter() {
                    classes_to_init.push(to_initialize)
                }
            }
        }
        classes_to_init.push(class_ref);

        debug!(
            "initializing class {}, classes to init {:?}",
            class_ref.name,
            classes_to_init
                .iter()
                .map(|c| &c.name)
                .collect::<Vec<&String>>()
        );

        Ok(ClassesToInitialize {
            resolved_class: class_ref,
            to_initialize: classes_to_init,
        })
    }

    fn new_class(
        class_file: ClassFile,
        id: ClassId,
        resolved_classes: &IndexMap<String, ResolvedClass<'a>>,
    ) -> Result<Class<'a>, VmError> {
        let superclass = class_file
            .superclass
            .as_ref()
            .map(|superclass_name| resolved_classes.get(superclass_name).unwrap().get_class());
        let interfaces: Vec<ClassRef<'a>> = class_file
            .interfaces
            .iter()
            .map(|interface_name| resolved_classes.get(interface_name).unwrap().get_class())
            .collect();

        let num_superclass_fields = match superclass {
            Some(superclass) => superclass.num_total_fields,
            None => 0,
        };
        let num_this_class_fields = class_file.fields.len();

        Ok(Class {
            id,
            name: class_file.name,
            source_file: class_file.source_file,
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
