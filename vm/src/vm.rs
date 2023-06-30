use std::{collections::HashMap, string::ToString};

use log::{debug, error, info};
use typed_arena::Arena;

use rjvm_reader::type_conversion::ToUsizeSafe;

use crate::{
    abstract_object::{AbstractObject, ObjectKind},
    array::Array,
    array_entry_type::ArrayEntryType,
    call_frame::MethodCallResult,
    call_stack::CallStack,
    class::{ClassId, ClassRef},
    class_and_method::ClassAndMethod,
    class_manager::{ClassManager, ResolvedClass},
    class_path::ClassPathParseError,
    class_resolver_by_id::ClassByIdResolver,
    exceptions::MethodCallFailed,
    gc::ObjectAllocator,
    native_methods_impl::array_copy,
    native_methods_registry::NativeMethodsRegistry,
    stack_trace_element::StackTraceElement,
    value::Value,
    vm_error::VmError,
};

/// An instance of the virtual machine. Single-threaded, can execute one method (generally `main`).
pub struct Vm<'a> {
    /// Responsible for allocating and storing classes
    class_manager: ClassManager<'a>,

    /// Responsible for allocating objects
    object_allocator: ObjectAllocator<'a>,

    /// Allocated call stacks
    call_stacks: Arena<CallStack<'a>>,

    /// To model static fields, we will create one special instance of each class
    /// and we will store it in this map. This is a bit hacky, and wastes memory
    /// because we will allocate space for non-static fields, but it works easily!
    statics: HashMap<ClassId, AbstractObject<'a>>,

    /// Stores native methods
    pub native_methods_registry: NativeMethodsRegistry<'a>,

    /// Stores call stacks collected, and associate them with their throwable.
    /// In the classes that we are using, the Throwable implementation does not
    /// store the stack trace in the java fields, but rather relies on a native
    /// array. Since we have no place to store it inside the actual object, we will
    /// keep it in this weird map.
    /// See the implementation of Throwable::getStackTrace() in our rt.jar for
    /// clarity.
    throwable_call_stacks: HashMap<i32, Vec<StackTraceElement<'a>>>,

    /// Since we do not have I/O, we have a fake native method that does a println.
    /// To check in the tests what the java bytecode printed, we store it here.
    pub printed: Vec<Value<'a>>,
}

pub const ONE_MEGABYTE: usize = 1024 * 1024;
const DEFAULT_MAX_MB_OF_MEMORY: usize = 100;
pub const DEFAULT_MAX_MEMORY: usize = 100 * ONE_MEGABYTE;
pub const DEFAULT_MAX_MEMORY_MB_STR: &str = const_format::formatcp!("{}", DEFAULT_MAX_MB_OF_MEMORY);

impl<'a> ClassByIdResolver<'a> for Vm<'a> {
    fn find_class_by_id(&self, class_id: ClassId) -> Option<ClassRef<'a>> {
        self.class_manager.find_class_by_id(class_id)
    }
}

impl<'a> Vm<'a> {
    pub fn new(max_memory: usize) -> Self {
        info!("Creating new VM with maximum memory {}", max_memory);
        let mut result = Self {
            class_manager: Default::default(),
            object_allocator: ObjectAllocator::with_maximum_memory(max_memory),
            call_stacks: Arena::new(),
            statics: Default::default(),
            native_methods_registry: Default::default(),
            throwable_call_stacks: Default::default(),
            printed: Vec::new(),
        };
        crate::native_methods_impl::register_natives(&mut result.native_methods_registry);
        result
    }

    pub(crate) fn get_static_instance(&self, class_id: ClassId) -> Option<AbstractObject<'a>> {
        self.statics.get(&class_id).cloned()
    }

    pub fn append_class_path(&mut self, class_path: &str) -> Result<(), ClassPathParseError> {
        self.class_manager.append_class_path(class_path)
    }

    pub fn get_or_resolve_class(
        &mut self,
        stack: &mut CallStack<'a>,
        class_name: &str,
    ) -> Result<ClassRef<'a>, MethodCallFailed<'a>> {
        let class = self.class_manager.get_or_resolve_class(class_name)?;
        if let ResolvedClass::NewClass(classes_to_init) = &class {
            for class_to_init in classes_to_init.to_initialize.iter() {
                self.init_class(stack, class_to_init)?;
            }
        }
        Ok(class.get_class())
    }

    fn init_class(
        &mut self,
        stack: &mut CallStack<'a>,
        class_to_init: &ClassRef<'a>,
    ) -> Result<(), MethodCallFailed<'a>> {
        debug!("creating static instance of {}", class_to_init.name);
        let static_instance = self.new_object_of_class(class_to_init);
        self.statics.insert(class_to_init.id, static_instance);
        if let Some(clinit_method) = class_to_init.find_method("<clinit>", "()V") {
            debug!("invoking {}::<clinit>()", class_to_init.name);
            self.invoke(
                stack,
                ClassAndMethod {
                    class: class_to_init,
                    method: clinit_method,
                },
                None,
                Vec::new(),
            )?;
        }
        Ok(())
    }

    pub fn get_class_by_id(&self, class_id: ClassId) -> Result<ClassRef<'a>, VmError> {
        self.find_class_by_id(class_id)
            .ok_or(VmError::ValidationException)
    }

    pub fn find_class_by_name(&self, class_name: &str) -> Option<ClassRef<'a>> {
        self.class_manager.find_class_by_name(class_name)
    }

    pub fn resolve_class_method(
        &mut self,
        call_stack: &mut CallStack<'a>,
        class_name: &str,
        method_name: &str,
        method_type_descriptor: &str,
    ) -> Result<ClassAndMethod<'a>, MethodCallFailed<'a>> {
        self.get_or_resolve_class(call_stack, class_name)
            .and_then(|class| {
                class
                    .find_method(method_name, method_type_descriptor)
                    .map(|method| ClassAndMethod { class, method })
                    .ok_or(MethodCallFailed::InternalError(
                        VmError::MethodNotFoundException(
                            class_name.to_string(),
                            method_name.to_string(),
                            method_type_descriptor.to_string(),
                        ),
                    ))
            })
    }

    pub fn invoke(
        &mut self,
        call_stack: &mut CallStack<'a>,
        class_and_method: ClassAndMethod<'a>,
        object: Option<AbstractObject<'a>>,
        args: Vec<Value<'a>>,
    ) -> MethodCallResult<'a> {
        if class_and_method.method.is_native() {
            return self.invoke_native(call_stack, class_and_method, object, args);
        }

        // Generic bytecode method
        let mut frame = call_stack.add_frame(class_and_method, object, args)?;
        let result = frame.as_mut().execute(self, call_stack);
        call_stack
            .pop_frame()
            .expect("should be able to pop the frame we just pushed");
        result
    }

    fn invoke_native(
        &mut self,
        call_stack: &mut CallStack<'a>,
        class_and_method: ClassAndMethod<'a>,
        object: Option<AbstractObject<'a>>,
        args: Vec<Value<'a>>,
    ) -> MethodCallResult<'a> {
        let native_callback = self.native_methods_registry.get_method(&class_and_method);
        if let Some(native_callback) = native_callback {
            debug!(
                "executing native method {}::{} {}",
                class_and_method.class.name,
                class_and_method.method.name,
                class_and_method.method.type_descriptor
            );
            native_callback(self, call_stack, object, args)
        } else {
            error!(
                "cannot resolve native method {}::{} {}",
                class_and_method.class.name,
                class_and_method.method.name,
                class_and_method.method.type_descriptor
            );
            Err(MethodCallFailed::InternalError(VmError::NotImplemented))
        }
    }

    /// Allocates a new call stack. We need to store it to be able to refer it later, for
    /// extracting the gc roots.
    pub fn allocate_call_stack(&mut self) -> &'a mut CallStack<'a> {
        let stack = self.call_stacks.alloc(CallStack::new());
        unsafe {
            let stack_ptr: *mut CallStack<'a> = stack;
            &mut *stack_ptr
        }
    }

    pub fn new_object(
        &mut self,
        call_stack: &mut CallStack<'a>,
        class_name: &str,
    ) -> Result<AbstractObject<'a>, MethodCallFailed<'a>> {
        let class = self.get_or_resolve_class(call_stack, class_name)?;
        Ok(self.new_object_of_class(class))
    }

    pub fn new_object_of_class(&mut self, class: ClassRef<'a>) -> AbstractObject<'a> {
        debug!("allocating new instance of {}", class.name);
        match self.object_allocator.allocate_object(class) {
            Some(object) => object,
            None => {
                self.run_garbage_collection()
                    .expect("could run garbage collection");
                self.object_allocator
                    .allocate_object(class)
                    .expect("cannot allocate object even after full garbage collection!")
            }
        }
    }

    pub fn new_array(
        &mut self,
        elements_type: ArrayEntryType,
        length: usize,
    ) -> AbstractObject<'a> {
        match self
            .object_allocator
            .allocate_array(elements_type.clone(), length)
        {
            Some(array) => array,
            None => {
                self.run_garbage_collection()
                    .expect("could run garbage collection");
                self.object_allocator
                    .allocate_array(elements_type, length)
                    .expect("cannot allocate array even after full garbage collection!")
            }
        }
    }

    pub fn clone_array(&mut self, value: Value<'a>) -> Result<Value<'a>, VmError> {
        match &value {
            Value::Object(array) if array.kind() == ObjectKind::Array => {
                let new_array =
                    self.new_array(array.elements_type(), array.len().into_usize_safe());
                array_copy(array, 0, &new_array, 0, array.len().into_usize_safe())?;
                Ok(Value::Object(new_array))
            }
            _ => Err(VmError::ValidationException),
        }
    }

    pub(crate) fn associate_stack_trace_with_throwable(
        &mut self,
        throwable: AbstractObject<'a>,
        call_stack: Vec<StackTraceElement<'a>>,
    ) {
        self.throwable_call_stacks
            .insert(throwable.identity_hash_code(), call_stack);
    }

    pub(crate) fn get_stack_trace_associated_with_throwable(
        &self,
        throwable: AbstractObject<'a>,
    ) -> Option<&Vec<StackTraceElement<'a>>> {
        self.throwable_call_stacks
            .get(&throwable.identity_hash_code())
    }

    pub fn debug_stats(&self) {
        debug!(
            "VM classes={:?} allocator={:?}",
            self.class_manager, self.object_allocator
        )
    }

    pub fn run_garbage_collection(&mut self) -> Result<(), VmError> {
        let mut roots = vec![];
        roots.extend(
            self.statics
                .iter_mut()
                .map(|(_, object)| object as *mut AbstractObject<'a>),
        );
        roots.extend(self.call_stacks.iter_mut().flat_map(|s| s.gc_roots()));

        unsafe {
            self.object_allocator
                .do_garbage_collection(roots, &self.class_manager)?;
        }
        Ok(())
    }
}
