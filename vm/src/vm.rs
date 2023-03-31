use log::{debug, info};

use crate::{
    call_stack::CallStack,
    class::{ClassId, ClassRef},
    class_and_method::ClassAndMethod,
    class_manager::ClassManager,
    class_path::ClassPathParseError,
    gc::ObjectAllocator,
    value::{ObjectRef, Value},
    vm_error::VmError,
};

#[derive(Debug, Default)]
pub struct Vm<'a> {
    class_manager: ClassManager<'a>,
    object_allocator: ObjectAllocator<'a>,
    pub printed: Vec<Value<'a>>, // Temporary, used for testing purposes
}

impl<'a> Vm<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn append_class_path(&mut self, class_path: &str) -> Result<(), ClassPathParseError> {
        self.class_manager.append_class_path(class_path)
    }

    pub fn get_or_resolve_class(&mut self, class_name: &str) -> Result<ClassRef<'a>, VmError> {
        self.class_manager.get_or_resolve_class(class_name)
    }

    pub fn find_class_by_id(&self, class_id: ClassId) -> Option<ClassRef<'a>> {
        self.class_manager.find_class_by_id(class_id)
    }

    pub fn resolve_class_method(
        &mut self,
        class_name: &str,
        method_name: &str,
        method_type_descriptor: &str,
    ) -> Result<ClassAndMethod<'a>, VmError> {
        self.get_or_resolve_class(class_name).and_then(|class| {
            class
                .find_method(method_name, method_type_descriptor)
                .map(|method| ClassAndMethod { class, method })
                .ok_or(VmError::ClassNotFoundException(class_name.to_string()))
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

        let class = self.get_or_resolve_class(class_name)?;
        let instance = self.object_allocator.allocate(class);
        Ok(instance)
    }

    pub fn debug_stats(&self) {
        debug!(
            "VM classes={:?}, objects = {:?}",
            self.class_manager, self.object_allocator
        )
    }
}
