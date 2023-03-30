use rjvm_vm::{value::Value, vm::Vm, vm_error::VmError};

fn ensure_class_is_resolved(vm: &mut Vm, class_name: &str) {
    vm.resolve_class(class_name)
        .unwrap_or_else(|_| panic!("should be able to load class {class_name}"));
}

fn create_base_vm() -> Vm<'static> {
    let mut vm = Vm::new();

    let resources_dir = env!("CARGO_MANIFEST_DIR");
    vm.append_class_path(&format!(
        "{resources_dir}/tests/resources:{resources_dir}/tests/resources/jre-8-rt",
    ))
    .expect("should be able to add entries to the classpath");
    ensure_class_is_resolved(&mut vm, "java/lang/Object");
    vm
}

fn invoke<'a>(
    vm: &mut Vm<'a>,
    class_name: &str,
    method_name: &str,
    descriptor: &str,
) -> Result<Option<Value<'a>>, VmError> {
    let main_method = vm
        .find_class_method(class_name, method_name, descriptor)
        .expect("should find main method");

    let mut call_stack = vm.allocate_call_stack();
    let main_result = vm.invoke(&mut call_stack, main_method, None, vec![]);
    vm.debug_stats();
    print!("result of {class_name}::{method_name}: {main_result:?}");

    main_result
}

#[test_log::test]
fn simple_main() {
    let mut vm = create_base_vm();
    ensure_class_is_resolved(&mut vm, "rjvm/SimpleMain");
    ensure_class_is_resolved(&mut vm, "rjvm/SimpleMain$Generator.class");

    let main_result = invoke(&mut vm, "rjvm/SimpleMain", "main", "([Ljava/lang/String;)V");
    assert_eq!(Ok(None), main_result);

    assert_eq!(vec![Value::Int(3), Value::Int(6)], vm.printed);
}

#[test_log::test]
fn superclasses() {
    let mut vm = create_base_vm();
    ensure_class_is_resolved(&mut vm, "rjvm/SuperClasses");
    ensure_class_is_resolved(&mut vm, "rjvm/SuperClasses$BaseClass");
    ensure_class_is_resolved(&mut vm, "rjvm/SuperClasses$DerivedClass");

    let main_result = invoke(
        &mut vm,
        "rjvm/SuperClasses",
        "main",
        "([Ljava/lang/String;)V",
    );
    assert_eq!(Ok(None), main_result);

    assert_eq!(vec![Value::Int(4)], vm.printed);
}

#[test_log::test]
fn control_flow() {
    let mut vm = create_base_vm();
    ensure_class_is_resolved(&mut vm, "rjvm/ControlFlow");

    let main_result = invoke(
        &mut vm,
        "rjvm/ControlFlow",
        "main",
        "([Ljava/lang/String;)V",
    );
    assert_eq!(Ok(None), main_result);

    assert_eq!(
        vec![
            Value::Int(241),
            Value::Int(42),
            Value::Int(1),
            Value::Int(1),
            Value::Int(1),
        ],
        vm.printed
    );
}

#[test_log::test]
fn numeric_types() {
    let mut vm = create_base_vm();
    ensure_class_is_resolved(&mut vm, "rjvm/NumericTypes");

    let main_result = invoke(
        &mut vm,
        "rjvm/NumericTypes",
        "main",
        "([Ljava/lang/String;)V",
    );
    assert_eq!(Ok(None), main_result);

    assert_eq!(
        vec![
            Value::Int(3),
            Value::Float(3.45f32),
            Value::Int(3),
            Value::Long(3),
            Value::Double(3.45f32 as f64),
            Value::Long(2),
            Value::Int(2),
            Value::Float(2f32),
            Value::Double(2f64),
            Value::Double(4.45),
            Value::Int(4),
            Value::Float(4.45),
            Value::Long(4),
            Value::Int(-1),
            Value::Long(-1),
            Value::Float(-1f32),
            Value::Double(-1f64),
            Value::Int(1),
            Value::Int(((-1i32) as u32 >> 2) as i32),
            Value::Int(8),
            Value::Long(1),
            Value::Long(((-1i64) as u64 >> 2) as i64),
            Value::Long(8),
        ],
        vm.printed
    );
}

#[test_log::test]
fn numeric_arrays() {
    let mut vm = create_base_vm();
    ensure_class_is_resolved(&mut vm, "rjvm/NumericArrays");

    let main_result = invoke(
        &mut vm,
        "rjvm/NumericArrays",
        "main",
        "([Ljava/lang/String;)V",
    );
    assert_eq!(Ok(None), main_result);

    assert_eq!(
        vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(0x03),
            Value::Int('b' as i32),
            Value::Int(-1),
            Value::Int(12),
            Value::Long(2),
            Value::Float(1.2f32 + 0.2f32),
            Value::Double(0f64),
        ],
        vm.printed
    );
}

#[test_log::test]
fn object_arrays() {
    let mut vm = create_base_vm();
    ensure_class_is_resolved(&mut vm, "rjvm/ObjectArrays");
    ensure_class_is_resolved(&mut vm, "rjvm/ObjectArrays$Square");

    let main_result = invoke(
        &mut vm,
        "rjvm/ObjectArrays",
        "main",
        "([Ljava/lang/String;)V",
    );
    assert_eq!(Ok(None), main_result);

    assert_eq!(vec![Value::Int(5),], vm.printed);
}
