use log::{debug, info};

use rjvm_reader::class_file::ClassFile;

use crate::{
    call_stack::CallStack,
    class::{ClassId, ClassRef},
    class_allocator::{ClassAllocator, ClassResolver},
    class_and_method::ClassAndMethod,
    class_loader::ClassLoader,
    gc::ObjectAllocator,
    value::{ObjectRef, Value},
    vm_error::VmError,
};

#[derive(Debug, Default)]
pub struct Vm<'a> {
    class_allocator: ClassAllocator<'a>,
    pub(crate) class_loader: ClassLoader<'a>,
    object_allocator: ObjectAllocator<'a>,
    pub printed: Vec<Value<'a>>, // Temporary, used for testing purposes
}

impl<'a> Vm<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn load_class(&mut self, class_file: ClassFile) -> Result<(), VmError> {
        let class = self
            .class_allocator
            .allocate(class_file, &self.class_loader)?;
        self.class_loader.register_class(class);
        Ok(())
    }

    pub fn find_class<'b>(&'b self, class_name: &str) -> Option<ClassRef<'a>> {
        self.class_loader.find_class_by_name(class_name)
    }

    pub fn get_class(&self, class_name: &str) -> Result<ClassRef<'a>, VmError> {
        self.find_class(class_name)
            .ok_or(VmError::ClassNotFoundException(class_name.to_string()))
    }

    pub fn get_class_by_id(&self, class_id: ClassId) -> Result<ClassRef<'a>, VmError> {
        self.class_loader
            .find_class_by_id(class_id)
            .ok_or(VmError::ClassNotFoundException(class_id.to_string()))
    }

    pub fn find_class_method(
        &self,
        class_name: &str,
        method_name: &str,
        method_type_descriptor: &str,
    ) -> Option<ClassAndMethod<'a>> {
        self.find_class(class_name).and_then(|class| {
            class
                .find_method(method_name, method_type_descriptor)
                .map(|method| ClassAndMethod { class, method })
        })
    }

    // TODO: do we need it?
    pub fn allocate_call_stack(&self) -> CallStack<'a> {
        CallStack::new()
    }

    pub fn invoke(
        &mut self,
        call_stack: &mut CallStack<'a>,
        class_and_method: ClassAndMethod<'a>,
        object: Option<ObjectRef<'a>>,
        args: Vec<Value<'a>>,
    ) -> Result<Option<Value<'a>>, VmError> {
        if class_and_method.method.is_native() {
            return if class_and_method.class.name.starts_with("rjvm/")
                && class_and_method.method.name == "tempPrint"
            {
                let arg = args.get(0).ok_or(VmError::ValidationException)?;
                info!("TEMP implementation of native method: printing value {arg:?}");
                self.printed.push(arg.clone());
                Ok(None)
            } else {
                Err(VmError::NotImplemented)
            };
        }

        let frame = call_stack.add_frame(class_and_method, object, args)?;
        let result = frame.borrow_mut().execute(self, call_stack);
        call_stack.pop_frame()?;
        result
    }

    pub fn new_object(&mut self, class_name: &str) -> Result<ObjectRef<'a>, VmError> {
        debug!("allocating new instance of {}", class_name);

        let class = self.get_class(class_name)?;
        let instance = self.object_allocator.allocate(class);
        Ok(instance)
    }

    pub fn debug_stats(&self) {
        debug!(
            "VM classes={:?}, objects = {:?}",
            self.class_allocator, self.object_allocator
        )
    }
}
