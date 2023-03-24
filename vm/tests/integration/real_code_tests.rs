use rjvm_reader::utils;
use rjvm_vm::value::Value;
use rjvm_vm::vm::Vm;
use rjvm_vm::vm_error::VmError;

fn load_class(vm: &mut Vm, bytes: &[u8]) {
    let class_file = utils::read_class_from_bytes(bytes);
    vm.load_class(class_file).unwrap();
}

fn create_base_vm() -> Vm<'static> {
    let mut vm = Vm::new();
    load_class(
        &mut vm,
        include_bytes!("../resources/jre-8-rt/java/lang/Object.class"),
    );
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

    let mut stack = vm.allocate_stack();
    let main_result = vm.invoke(&mut stack, main_method, None, vec![]);
    vm.debug_stats();
    print!("result of {class_name}::{method_name}: {main_result:?}");

    main_result
}

#[test_log::test]
fn simple_main() {
    let mut vm = create_base_vm();
    load_class(
        &mut vm,
        include_bytes!("../resources/rjvm/SimpleMain.class"),
    );
    load_class(
        &mut vm,
        include_bytes!("../resources/rjvm/SimpleMain$Generator.class"),
    );

    let main_result = invoke(&mut vm, "rjvm/SimpleMain", "main", "([Ljava/lang/String;)V");
    assert_eq!(Ok(None), main_result);

    assert_eq!(vec![Value::Int(3), Value::Int(6)], vm.printed);
}

#[test_log::test]
fn superclasses() {
    let mut vm = create_base_vm();
    load_class(
        &mut vm,
        include_bytes!("../resources/rjvm/SuperClasses.class"),
    );
    load_class(
        &mut vm,
        include_bytes!("../resources/rjvm/SuperClasses$BaseClass.class"),
    );
    load_class(
        &mut vm,
        include_bytes!("../resources/rjvm/SuperClasses$DerivedClass.class"),
    );

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
    load_class(
        &mut vm,
        include_bytes!("../resources/rjvm/ControlFlow.class"),
    );

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
    load_class(
        &mut vm,
        include_bytes!("../resources/rjvm/NumericTypes.class"),
    );

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
        ],
        vm.printed
    );
}

#[test_log::test]
fn numeric_arrays() {
    let mut vm = create_base_vm();
    load_class(
        &mut vm,
        include_bytes!("../resources/rjvm/NumericArrays.class"),
    );

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
    load_class(
        &mut vm,
        include_bytes!("../resources/rjvm/ObjectArrays.class"),
    );
    load_class(
        &mut vm,
        include_bytes!("../resources/rjvm/ObjectArrays$Square.class"),
    );

    let main_result = invoke(
        &mut vm,
        "rjvm/ObjectArrays",
        "main",
        "([Ljava/lang/String;)V",
    );
    assert_eq!(Ok(None), main_result);

    assert_eq!(vec![Value::Int(5),], vm.printed);
}
