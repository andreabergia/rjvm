use log::{debug, info};

use rjvm_utils::type_conversion::ToUsizeSafe;

use crate::{
    call_frame::MethodCallResult,
    call_stack::CallStack,
    native_methods_registry::NativeMethodsRegistry,
    time::{get_current_time_millis, get_nano_time},
    value::{
        expect_array_at, expect_double_at, expect_float_at, expect_int_at, expect_object_at,
        ObjectRef, Value,
    },
    vm::Vm,
    vm_error::VmError,
};

pub(crate) fn register_natives(registry: &mut NativeMethodsRegistry) {
    registry.register_temp_print(|vm, _, _, args| temp_print(vm, args));
    register_noops(registry);
    register_time_methods(registry);
    register_gc_methods(registry);
    register_native_repr_methods(registry);
    register_reflection_methods(registry);
}

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

fn register_gc_methods(registry: &mut NativeMethodsRegistry) {
    registry.register(
        "java/lang/System",
        "identityHashCode",
        "(Ljava/lang/Object;)I",
        |_, _, _, args| identity_hash_code(args),
    );
}

fn register_native_repr_methods(registry: &mut NativeMethodsRegistry) {
    registry.register(
        "java/lang/System",
        "arraycopy",
        "(Ljava/lang/Object;ILjava/lang/Object;II)V",
        |_, _, _, args| array_copy(args),
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

fn temp_print<'a>(vm: &mut Vm<'a>, args: Vec<Value<'a>>) -> MethodCallResult<'a> {
    let arg = args.get(0).ok_or(VmError::ValidationException)?;
    info!(
        "TEMP implementation of native method: printing value {:?}",
        args
    );
    vm.printed.push(arg.clone());
    Ok(None)
}

fn identity_hash_code<'a>(args: Vec<Value<'a>>) -> MethodCallResult<'a> {
    let object = expect_object_at(&args, 0)?;
    // TODO: we need some sort of object id when we implement the GC
    //  For the moment we'll use the raw address
    let ptr = &object as *const ObjectRef<'a>;
    let address: i32 = ptr as i32;
    Ok(Some(Value::Int(address)))
}

fn array_copy(args: Vec<Value>) -> MethodCallResult {
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

fn get_class_loader(receiver: Option<ObjectRef>) -> MethodCallResult {
    debug!(
        "invoked get class loader for class {:?}",
        receiver.map(|r| r.class_id)
    );

    // TODO: it seems ok to return just null for the moment
    Ok(Some(Value::Null))
}

fn get_primitive_class<'a>(
    vm: &mut Vm<'a>,
    stack: &mut CallStack<'a>,
    args: &[Value<'a>],
) -> MethodCallResult<'a> {
    let arg = expect_object_at(args, 0)?;
    let class_name = vm.extract_str_from_java_lang_string(arg)?;
    let java_lang_class_instance = vm.create_instance_of_java_lang_class(stack, &class_name)?;
    Ok(Some(Value::Object(java_lang_class_instance)))
}
