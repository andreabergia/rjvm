use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use log::{debug, info, warn};

use rjvm_reader::field_type::{BaseType, FieldType};
use rjvm_utils::type_conversion::ToUsizeSafe;

use crate::class_manager::ResolvedClass;
use crate::native_methods::NativeMethodsRegistry;
use crate::time::{get_current_time_millis, get_nano_time};
use crate::value::ArrayRef;
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
    /// Responsible for allocating and storing classes
    class_manager: ClassManager<'a>,

    /// Responsible for allocating objects
    object_allocator: ObjectAllocator<'a>,

    /// To model static fields, we will create one special instance of each class
    /// and we will store it in this map
    statics: HashMap<ClassId, ObjectRef<'a>>,

    /// Stores native methods
    pub native_methods_registry: NativeMethodsRegistry<'a>,

    pub printed: Vec<Value<'a>>, // Temporary, used for testing purposes
}

impl<'a> Vm<'a> {
    pub fn new() -> Self {
        let mut result: Self = Default::default();
        result.register_natives();
        result
    }

    fn register_natives(&mut self) {
        self.native_methods_registry
            .register_temp_print(|vm, _, _, _, args| {
                let arg = args.get(0).ok_or(VmError::ValidationException)?;
                info!(
                    "TEMP implementation of native method: printing value {:?}",
                    args
                );
                vm.printed.push(arg.clone());
                Ok(None)
            });

        self.native_methods_registry.register(
            "java/lang/Object",
            "registerNatives",
            "()V",
            |_, _, _, _, _| Ok(None),
        );
        self.native_methods_registry.register(
            "java/lang/System",
            "registerNatives",
            "()V",
            |_, _, _, _, _| Ok(None),
        );
        self.native_methods_registry.register(
            "java/lang/Class",
            "registerNatives",
            "()V",
            |_, _, _, _, _| Ok(None),
        );
        self.native_methods_registry.register(
            "java/lang/ClassLoader",
            "registerNatives",
            "()V",
            |_, _, _, _, _| Ok(None),
        );
        self.native_methods_registry.register(
            "java/lang/System",
            "nanoTime",
            "()J",
            |_, _, _, _, _| Ok(Some(Value::Long(get_nano_time()))),
        );
        self.native_methods_registry.register(
            "java/lang/System",
            "currentTimeMillis",
            "()J",
            |_, _, _, _, _| Ok(Some(Value::Long(get_current_time_millis()))),
        );
        self.native_methods_registry.register(
            "java/lang/System",
            "identityHashCode",
            "(Ljava/lang/Object;)I",
            |_, _, _, _, args| identity_hash_code(args),
        );
        self.native_methods_registry.register(
            "java/lang/System",
            "arraycopy",
            "(Ljava/lang/Object;ILjava/lang/Object;II)V",
            |_, _, _, _, args| array_copy(args),
        );
        self.native_methods_registry.register(
            "java/lang/Class",
            "getClassLoader0",
            "()Ljava/lang/ClassLoader;",
            |vm, call_stack, _, receiver, _| {
                debug!(
                    "invoked get class loader for class {:?}",
                    receiver.map(|r| r.class_id)
                );
                vm.create_class_loader_instance(call_stack)
                    .map(|v| Some(Value::Object(v)))
            },
        );
    }

    pub(crate) fn get_static_instance(&self, class_id: ClassId) -> Option<ObjectRef<'a>> {
        self.statics.get(&class_id).cloned()
    }

    pub fn append_class_path(&mut self, class_path: &str) -> Result<(), ClassPathParseError> {
        self.class_manager.append_class_path(class_path)
    }

    pub fn get_or_resolve_class(
        &mut self,
        stack: &mut CallStack<'a>,
        class_name: &str,
    ) -> Result<ClassRef<'a>, VmError> {
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
    ) -> Result<(), VmError> {
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

    pub fn find_class_by_id(&self, class_id: ClassId) -> Option<ClassRef<'a>> {
        self.class_manager.find_class_by_id(class_id)
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
    ) -> Result<ClassAndMethod<'a>, VmError> {
        self.get_or_resolve_class(call_stack, class_name)
            .and_then(|class| {
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
            let native_callback = self.native_methods_registry.get_method(&class_and_method);
            return if let Some(native_callback) = native_callback {
                debug!(
                    "executing native method {}::{} {}",
                    class_and_method.class.name,
                    class_and_method.method.name,
                    class_and_method.method.type_descriptor
                );
                native_callback(self, call_stack, class_and_method, object, args)
            } else {
                warn!(
                    "cannot resolve native method {}::{} {}",
                    class_and_method.class.name,
                    class_and_method.method.name,
                    class_and_method.method.type_descriptor
                );
                Err(VmError::NotImplemented)
            };
        }

        let frame = call_stack.add_frame(class_and_method, object, args)?;
        let result = frame.borrow_mut().execute(self, call_stack);
        call_stack.pop_frame()?;
        result
    }

    pub fn new_object(
        &mut self,
        call_stack: &mut CallStack<'a>,
        class_name: &str,
    ) -> Result<ObjectRef<'a>, VmError> {
        let class = self.get_or_resolve_class(call_stack, class_name)?;
        Ok(self.new_object_of_class(class))
    }

    pub fn new_object_of_class(&mut self, class: ClassRef<'a>) -> ObjectRef<'a> {
        debug!("allocating new instance of {}", class.name);
        self.object_allocator.allocate(class)
    }

    pub fn create_string_instance(
        &mut self,
        call_stack: &mut CallStack<'a>,
        string: &str,
    ) -> Result<ObjectRef<'a>, VmError> {
        let char_array: Vec<Value<'a>> = string
            .encode_utf16()
            .map(|c| Value::Int(c as i32))
            .collect();
        let char_array = Rc::new(RefCell::new(char_array));
        let char_array = Value::Array(FieldType::Base(BaseType::Char), char_array);

        // In our JRE's rt.jar, the fields for String are:
        //    private final char[] value;
        //    private int hash;
        //    private static final long serialVersionUID = -6849794470754667710L;
        //    private static final ObjectStreamField[] serialPersistentFields = new ObjectStreamField[0];
        //    public static final Comparator<String> CASE_INSENSITIVE_ORDER = new CaseInsensitiveComparator();
        //    private static final int HASHING_SEED;
        //    private transient int hash32;
        let string_object = self.new_object(call_stack, "java/lang/String")?;
        string_object.set_field(0, char_array);
        string_object.set_field(1, Value::Int(0));
        string_object.set_field(6, Value::Int(0));
        Ok(string_object)
    }

    pub fn create_class_instance(
        &mut self,
        call_stack: &mut CallStack<'a>,
        class_name: &str,
    ) -> Result<ObjectRef<'a>, VmError> {
        let class_object = self.new_object(call_stack, "java/lang/Class")?;
        // TODO: we should init the various fields...
        let string_object = Self::create_string_instance(self, call_stack, class_name)?;
        class_object.set_field(5, Value::Object(string_object));
        Ok(class_object)
    }

    pub fn create_class_loader_instance(
        &mut self,
        call_stack: &mut CallStack<'a>,
    ) -> Result<ObjectRef<'a>, VmError> {
        let class_loader_object = self.new_object(call_stack, "java/lang/ClassLoader")?;
        // TODO: we should init the various fields...
        Ok(class_loader_object)
    }

    pub fn debug_stats(&self) {
        debug!(
            "VM classes={:?}, objects = {:?}",
            self.class_manager, self.object_allocator
        )
    }
}

fn identity_hash_code<'a>(args: Vec<Value<'a>>) -> Result<Option<Value<'a>>, VmError> {
    let object = expect_object_at(&args, 0)?;
    // TODO: we need some sort of object id when we implement the GC
    //  For the moment we'll use the raw address
    let ptr = &object as *const ObjectRef<'a>;
    let address: i32 = ptr as i32;
    Ok(Some(Value::Int(address)))
}

fn array_copy(args: Vec<Value>) -> Result<Option<Value>, VmError> {
    let (_src_type, src) = expect_array_at(&args, 0)?;
    let src_pos = expect_int_at(&args, 1)?;
    let (_dest_type, dest) = expect_array_at(&args, 2)?;
    let dest_pos = expect_int_at(&args, 3)?;
    let length = expect_int_at(&args, 4)?;

    // TODO: handle NullPointerException
    // TODO: validate coherence of arrays types, or throw ArrayStoreException
    // TODO: validate length and indexes, or throw IndexOutOfBoundsException

    for i in 0..length {
        let src_index = (src_pos + i).into_usize_safe();
        let dest_index = (dest_pos + i).into_usize_safe();
        dest.borrow_mut()[dest_index] = src.borrow()[src_index].clone();
    }

    Ok(None)
}

fn expect_object_at<'a>(vec: &Vec<Value<'a>>, index: usize) -> Result<ObjectRef<'a>, VmError> {
    let value = vec.get(index);
    if let Some(Value::Object(object)) = value {
        Ok(object)
    } else {
        Err(VmError::ValidationException)
    }
}

fn expect_array_at<'a, 'b>(
    vec: &'b Vec<Value<'a>>,
    index: usize,
) -> Result<(&'b FieldType, &'b ArrayRef<'a>), VmError> {
    let value = vec.get(index);
    if let Some(Value::Array(field_type, array_ref)) = value {
        Ok((field_type, array_ref))
    } else {
        Err(VmError::ValidationException)
    }
}

fn expect_int_at(vec: &Vec<Value>, index: usize) -> Result<i32, VmError> {
    let value = vec.get(index);
    if let Some(Value::Int(int)) = value {
        Ok(*int)
    } else {
        Err(VmError::ValidationException)
    }
}
