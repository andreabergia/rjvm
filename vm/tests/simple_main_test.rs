use rjvm_reader::utils;
use rjvm_vm::vm::Vm;

fn load_class<'a>(vm: &mut Vm<'a>, bytes: &[u8]) {
    let class_file = utils::read_class_from_bytes(bytes);
    vm.load_class(class_file).unwrap();
}

#[test_log::test]
fn can_execute_real_code() {
    let mut vm = Vm::new();
    load_class(
        &mut vm,
        include_bytes!("resources/jre-8-rt/java/lang/Object.class"),
    );
    load_class(&mut vm, include_bytes!("resources/rjvm/SimpleMain.class"));
    load_class(
        &mut vm,
        include_bytes!("resources/rjvm/SimpleMain$Generator.class"),
    );

    let main_method = vm
        .find_class_method("rjvm/SimpleMain", "main", "([Ljava/lang/String;)V")
        .expect("should find main method");

    let mut stack = vm.allocate_stack();
    let main_result = vm.invoke(&mut stack, main_method, None, vec![]);
    print!("result: {main_result:?}");
    assert!(main_result.is_ok());
    assert!(main_result.unwrap().is_none());
}
