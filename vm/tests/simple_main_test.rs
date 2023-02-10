extern crate rjvm_vm;

use rjvm_reader::utils;
use rjvm_vm::vm::Vm;

trait LoadClass {
    fn load_class_from_bytes(&mut self, bytes: &[u8]);
}

impl LoadClass for Vm {
    fn load_class_from_bytes(&mut self, bytes: &[u8]) {
        self.load_class(utils::read_class_from_bytes(bytes))
    }
}

#[test_log::test]
fn can_execute_real_code() {
    let mut vm = Vm::new();
    vm.load_class_from_bytes(include_bytes!("resources/rjvm/SimpleMain.class"));
    vm.load_class_from_bytes(include_bytes!("resources/rjvm/SimpleMain$Generator.class"));
    vm.load_class_from_bytes(include_bytes!("resources/jre-8-rt/java/lang/Object.class"));

    let main_method = vm
        .find_class_method("rjvm/SimpleMain", "main", "([Ljava/lang/String;)V")
        .expect("should find main method");

    let mut stack = vm.allocate_stack();
    let main_result = vm.invoke(&mut stack, main_method, None, vec![]);
    print!("result: {main_result:?}");
    assert!(main_result.is_ok());
    assert!(main_result.unwrap().is_none());
}
