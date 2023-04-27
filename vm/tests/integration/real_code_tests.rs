use rjvm_vm::{
    exceptions::MethodCallFailed,
    value::{expect_object_at, Value},
    vm::Vm,
};

fn create_base_vm() -> Vm<'static> {
    let mut vm = Vm::new();

    let src_dir = env!("CARGO_MANIFEST_DIR");
    vm.append_class_path(&format!("{src_dir}/rt.jar:{src_dir}/tests/resources",))
        .expect("should be able to add entries to the classpath");
    vm
}

fn invoke<'a>(
    vm: &mut Vm<'a>,
    class_name: &str,
    method_name: &str,
    descriptor: &str,
) -> Result<Option<Value<'a>>, MethodCallFailed<'a>> {
    let mut call_stack = vm.allocate_call_stack();
    let main_method = vm
        .resolve_class_method(&mut call_stack, class_name, method_name, descriptor)
        .expect("should find main method");

    let main_result = vm.invoke(&mut call_stack, main_method, None, vec![]);
    vm.debug_stats();
    print!("result of {class_name}::{method_name}: {main_result:?}");

    main_result
}

#[test_log::test]
fn simple_main() {
    let mut vm = create_base_vm();
    let main_result = invoke(&mut vm, "rjvm/SimpleMain", "main", "([Ljava/lang/String;)V");
    assert_eq!(Ok(None), main_result);

    assert_eq!(vec![Value::Int(3), Value::Int(6)], vm.printed);
}

#[test_log::test]
fn superclasses() {
    let mut vm = create_base_vm();
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
    let main_result = invoke(
        &mut vm,
        "rjvm/ControlFlow",
        "main",
        "([Ljava/lang/String;)V",
    );
    assert_eq!(Ok(None), main_result);

    assert_eq!(
        vec![
            // Ints
            Value::Int(241),
            // Objects
            Value::Int(42),
            Value::Int(43),
            // Double
            Value::Int(1),
            Value::Int(1),
            Value::Int(1),
            // Arrays
            Value::Int(51),
            Value::Int(52),
        ],
        vm.printed
    );
}

#[test_log::test]
fn numeric_types() {
    let mut vm = create_base_vm();
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
    let main_result = invoke(
        &mut vm,
        "rjvm/ObjectArrays",
        "main",
        "([Ljava/lang/String;)V",
    );
    assert_eq!(Ok(None), main_result);

    assert_eq!(vec![Value::Int(5),], vm.printed);
}

#[test_log::test]
fn statics() {
    let mut vm = create_base_vm();
    let main_result = invoke(&mut vm, "rjvm/Statics", "main", "([Ljava/lang/String;)V");
    assert_eq!(Ok(None), main_result);

    assert_eq!(vec![Value::Int(311), Value::Int(322),], vm.printed);
}

#[test_log::test]
fn instance_of() {
    let mut vm = create_base_vm();
    let main_result = invoke(&mut vm, "rjvm/InstanceOf", "main", "([Ljava/lang/String;)V");
    assert_eq!(Ok(None), main_result);

    assert_eq!(
        vec![
            Value::Int(1),
            Value::Int(1),
            // C1
            Value::Int(0),
            Value::Int(0),
            Value::Int(0),
            Value::Int(0),
            // C2
            Value::Int(1),
            Value::Int(0),
            Value::Int(0),
            Value::Int(0),
            // C3
            Value::Int(1),
            Value::Int(1),
            Value::Int(0),
            Value::Int(0),
            // C4
            Value::Int(0),
            Value::Int(0),
            Value::Int(1),
            Value::Int(1),
            // C5
            Value::Int(1),
            Value::Int(0),
            Value::Int(1),
            Value::Int(1),
        ],
        vm.printed
    );
}

#[test_log::test]
fn instance_of_array() {
    let mut vm = create_base_vm();
    let main_result = invoke(
        &mut vm,
        "rjvm/InstanceOfArray",
        "main",
        "([Ljava/lang/String;)V",
    );
    assert_eq!(Ok(None), main_result);

    assert_eq!(
        vec![
            // C0
            Value::Int(1),
            Value::Int(1),
            Value::Int(0),
            Value::Int(0),
            // C1
            Value::Int(1),
            Value::Int(0),
            Value::Int(1),
            Value::Int(1),
        ],
        vm.printed
    );
}

#[test_log::test]
fn strings() {
    let mut vm = create_base_vm();
    let main_result = invoke(&mut vm, "rjvm/Strings", "main", "([Ljava/lang/String;)V");
    assert_eq!(Ok(None), main_result);

    assert_eq!(1, vm.printed.len());
    let string = expect_object_at(&vm.printed, 0).expect("should have printed an object");
    let string = vm
        .extract_str_from_java_lang_string(string)
        .expect("should have a valid string");
    assert_eq!("Hello, Andrea, you were born in 1985", string);
}

#[test_log::test]
fn invoke_interface() {
    let mut vm = create_base_vm();
    let main_result = invoke(
        &mut vm,
        "rjvm/InvokeInterface",
        "main",
        "([Ljava/lang/String;)V",
    );
    assert_eq!(Ok(None), main_result);

    assert_eq!(
        vec![Value::Int(12), Value::Int(4), Value::Int(10)],
        vm.printed
    );
}

#[test_log::test]
fn check_cast() {
    let mut vm = create_base_vm();
    let main_result = invoke(&mut vm, "rjvm/CheckCast", "main", "([Ljava/lang/String;)V");
    assert_eq!(Ok(None), main_result);

    assert_eq!(vec![Value::Int(1)], vm.printed);
}
