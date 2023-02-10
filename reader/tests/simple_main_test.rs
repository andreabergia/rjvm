extern crate rjvm_reader;

mod utils;

#[test_log::test]
fn can_execute_real_code() {
    let mut vm = rjvm_reader::vm::Vm::new();
    vm.load_class(utils::read_class_from_file("rjvm/SimpleMain"));
    vm.load_class(utils::read_class_from_file("rjvm/SimpleMain$Generator"));
    vm.load_class(utils::read_class_from_file("jre-8-rt/java/lang/Object"));

    let main_method = vm
        .find_class_method("rjvm/SimpleMain", "main", "([Ljava/lang/String;)V")
        .expect("should find main method");

    let mut stack = vm.allocate_stack();
    let main_result = vm.invoke(&mut stack, main_method, None, vec![]);
    print!("result: {main_result:?}");
    assert!(main_result.is_ok());
    assert!(main_result.unwrap().is_none());
}
