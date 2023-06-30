use log::{debug, info};

use rjvm_utils::type_conversion::ToUsizeSafe;

use crate::{
    abstract_object::{AbstractObject, ObjectKind},
    array::Array,
    call_frame::MethodCallResult,
    call_stack::CallStack,
    exceptions::MethodCallFailed,
    java_objects_creation::{
        extract_str_from_java_lang_string, new_java_lang_class_object,
        new_java_lang_stack_trace_element_object,
    },
    native_methods_registry::NativeMethodsRegistry,
    object::Object,
    time::{get_current_time_millis, get_nano_time},
    value::{
        expect_abstract_object_at, expect_array_at, expect_concrete_object_at, expect_double_at,
        expect_float_at, expect_int_at, Value,
    },
    vm::Vm,
    vm_error::VmError,
};

/// Registers the built-in native methods
pub(crate) fn register_natives(registry: &mut NativeMethodsRegistry) {
    registry.register_temp_print(|vm, _, _, args| temp_print(vm, args));
    register_noops(registry);
    register_time_methods(registry);
    register_gc_methods(registry);
    register_native_repr_methods(registry);
    register_reflection_methods(registry);
    register_throwable_methods(registry);
}

/// These various methods are noop, i.e. they do not do anything
fn register_noops(registry: &mut NativeMethodsRegistry) {
    registry.register(
        "java/lang/Object",
        "registerNatives",
        "()V",
        |_, _, _, _| Ok(None),
    );
    registry.register(
        "java/lang/System",
        "registerNatives",
        "()V",
        |_, _, _, _| Ok(None),
    );
    registry.register("java/lang/Class", "registerNatives", "()V", |_, _, _, _| {
        Ok(None)
    });
    registry.register(
        "java/lang/ClassLoader",
        "registerNatives",
        "()V",
        |_, _, _, _| Ok(None),
    );
}

/// Methods to access the system clock
fn register_time_methods(registry: &mut NativeMethodsRegistry) {
    registry.register("java/lang/System", "nanoTime", "()J", |_, _, _, _| {
        Ok(Some(Value::Long(get_nano_time())))
    });
    registry.register(
        "java/lang/System",
        "currentTimeMillis",
        "()J",
        |_, _, _, _| Ok(Some(Value::Long(get_current_time_millis()))),
    );
}

/// Methods related to the garbage collector
fn register_gc_methods(registry: &mut NativeMethodsRegistry) {
    registry.register(
        "java/lang/System",
        "identityHashCode",
        "(Ljava/lang/Object;)I",
        |_, _, _, args| identity_hash_code(args),
    );
    registry.register("java/lang/System", "gc", "()V", |vm, _, _, _| {
        vm.run_garbage_collection()?;
        Ok(None)
    });
}

/// Native methods that deal with the internal representation of data
fn register_native_repr_methods(registry: &mut NativeMethodsRegistry) {
    registry.register(
        "java/lang/System",
        "arraycopy",
        "(Ljava/lang/Object;ILjava/lang/Object;II)V",
        |_, _, _, args| native_array_copy(args),
    );
    registry.register(
        "java/lang/Float",
        "floatToRawIntBits",
        "(F)I",
        |_, _, _, args| float_to_raw_int_bits(&args),
    );
    registry.register(
        "java/lang/Double",
        "doubleToRawLongBits",
        "(D)J",
        |_, _, _, args| double_to_raw_long_bits(&args),
    );
}

/// Methods related to reflection
fn register_reflection_methods(registry: &mut NativeMethodsRegistry) {
    registry.register(
        "java/lang/Class",
        "getClassLoader0",
        "()Ljava/lang/ClassLoader;",
        |_, _, receiver, _| get_class_loader(receiver),
    );
    registry.register(
        "java/lang/Class",
        "desiredAssertionStatus0",
        "(Ljava/lang/Class;)Z",
        |_, _, _, _| Ok(Some(Value::Int(1))),
    );
    registry.register(
        "java/lang/Class",
        "getPrimitiveClass",
        "(Ljava/lang/String;)Ljava/lang/Class;",
        |vm, stack, _, args| get_primitive_class(vm, stack, &args),
    );
}

/// Methods of java.lang.Throwable
fn register_throwable_methods(registry: &mut NativeMethodsRegistry) {
    registry.register(
        "java/lang/Throwable",
        "fillInStackTrace",
        "(I)Ljava/lang/Throwable;",
        |vm, call_stack, receiver, _| fill_in_stack_trace(vm, call_stack, receiver),
    );
    registry.register(
        "java/lang/Throwable",
        "getStackTraceDepth",
        "()I",
        |vm, _, receiver, _| get_stack_trace_depth(vm, receiver),
    );
    registry.register(
        "java/lang/Throwable",
        "getStackTraceElement",
        "(I)Ljava/lang/StackTraceElement;",
        get_stack_trace_element,
    );
}

/// Debug method that does a "println", useful since we do not have real I/O
fn temp_print<'a>(vm: &mut Vm<'a>, args: Vec<Value<'a>>) -> MethodCallResult<'a> {
    let arg = args.get(0).ok_or(VmError::ValidationException)?;

    let formatted = match arg {
        Value::Object(object) if object.kind() == ObjectKind::Object => {
            let class = vm
                .get_class_by_id(object.class_id())
                .expect("cannot get an object without a valid class id");
            if class.name == "java/lang/String" {
                extract_str_from_java_lang_string(vm, object)
                    .expect("should be able to get a string's content")
            } else {
                format!("{:?}", object)
            }
        }
        _ => format!("{:?}", arg),
    };
    info!("TEMP implementation of native method: printing value {formatted}",);
    vm.printed.push(arg.clone());
    Ok(None)
}

fn identity_hash_code(args: Vec<Value<'_>>) -> MethodCallResult<'_> {
    let object = expect_abstract_object_at(&args, 0)?;
    Ok(Some(Value::Int(object.identity_hash_code())))
}

fn native_array_copy(args: Vec<Value>) -> MethodCallResult {
    // TODO: handle NullPointerException with the correct error

    let src = expect_array_at(&args, 0)?;
    let src_pos = expect_int_at(&args, 1)?;
    let dest = expect_array_at(&args, 2)?;
    let dest_pos = expect_int_at(&args, 3)?;
    let length = expect_int_at(&args, 4)?;
    array_copy(&src, src_pos, &dest, dest_pos, length.into_usize_safe())?;
    Ok(None)
}

pub fn array_copy<'a>(
    src: &impl Array<'a>,
    src_pos: i32,
    dest: &impl Array<'a>,
    dest_pos: i32,
    length: usize,
) -> Result<(), VmError> {
    if dest.elements_type() != src.elements_type() {
        // TODO: we should throw an instance of ArrayStoreException
        return Err(VmError::ValidationException);
    }

    for i in 0..length {
        let src_index = src_pos.into_usize_safe() + i;
        let src_item = src.get_element(src_index)?;

        let dest_index = dest_pos.into_usize_safe() + i;
        dest.set_element(dest_index, src_item)?;
    }

    Ok(())
}

fn float_to_raw_int_bits<'a>(args: &[Value<'a>]) -> MethodCallResult<'a> {
    let arg = expect_float_at(args, 0)?;
    let int_bits: i32 = arg.to_bits() as i32;
    Ok(Some(Value::Int(int_bits)))
}

fn double_to_raw_long_bits<'a>(args: &[Value<'a>]) -> MethodCallResult<'a> {
    let arg = expect_double_at(args, 0)?;
    let long_bits: i64 = arg.to_bits() as i64;
    Ok(Some(Value::Long(long_bits)))
}

fn get_class_loader(receiver: Option<AbstractObject>) -> MethodCallResult {
    debug!("invoked get class loader for object {:?}", receiver);

    // It seems ok to return just null for the moment
    Ok(Some(Value::Null))
}

fn get_primitive_class<'a>(
    vm: &mut Vm<'a>,
    stack: &mut CallStack<'a>,
    args: &[Value<'a>],
) -> MethodCallResult<'a> {
    let arg = expect_concrete_object_at(args, 0)?;
    let class_name = extract_str_from_java_lang_string(vm, &arg)?;
    let java_lang_class_instance = new_java_lang_class_object(vm, stack, &class_name)?;
    Ok(Some(Value::Object(java_lang_class_instance)))
}

fn fill_in_stack_trace<'a>(
    vm: &mut Vm<'a>,
    call_stack: &mut CallStack<'a>,
    receiver: Option<AbstractObject<'a>>,
) -> MethodCallResult<'a> {
    let receiver = expect_some_receiver(receiver)?;
    let stack_trace_elements = call_stack.get_stack_trace_elements();
    vm.associate_stack_trace_with_throwable(receiver.clone(), stack_trace_elements);
    Ok(Some(Value::Object(receiver)))
}

fn get_stack_trace_depth<'a>(
    vm: &mut Vm<'a>,
    receiver: Option<AbstractObject<'a>>,
) -> MethodCallResult<'a> {
    let receiver = expect_some_receiver(receiver)?;
    match vm.get_stack_trace_associated_with_throwable(receiver) {
        Some(stack_trace_elements) => Ok(Some(Value::Int(stack_trace_elements.len() as i32))),
        None => Err(MethodCallFailed::InternalError(
            VmError::ValidationException,
        )),
    }
}

fn get_stack_trace_element<'a>(
    vm: &mut Vm<'a>,
    call_stack: &mut CallStack<'a>,
    receiver: Option<AbstractObject<'a>>,
    args: Vec<Value<'a>>,
) -> MethodCallResult<'a> {
    let receiver = expect_some_receiver(receiver)?;
    let index = expect_int_at(&args, 0)?;
    match vm.get_stack_trace_associated_with_throwable(receiver) {
        Some(stack_trace_elements) => {
            let stack_trace_element = &stack_trace_elements[index.into_usize_safe()].clone();
            let stack_trace_element_java_object =
                new_java_lang_stack_trace_element_object(vm, call_stack, stack_trace_element)?;
            Ok(Some(Value::Object(stack_trace_element_java_object)))
        }
        None => Err(MethodCallFailed::InternalError(
            VmError::ValidationException,
        )),
    }
}

fn expect_some_receiver(receiver: Option<AbstractObject>) -> Result<AbstractObject, VmError> {
    match receiver {
        Some(v) => Ok(v),
        None => Err(VmError::ValidationException),
    }
}
