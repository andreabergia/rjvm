extern crate rjvm;

use rjvm::vm::class_and_method::ClassAndMethod;

mod utils;

#[test_log::test]
fn can_read_simple_main() {
    let class_main = utils::read_class_from_file("SimpleMain");
    let class_generator = utils::read_class_from_file("SimpleMain$Generator");

    let mut vm = rjvm::vm::Vm::new();
    vm.load_class(class_main);
    vm.load_class(class_generator);

    let main_method = vm.find_class("rjvm/SimpleMain").and_then(|class| {
        class
            .find_method("main", "([Ljava/lang/String;)V")
            .map(|method| ClassAndMethod { class, method })
    });
    assert!(main_method.is_some());
    let main_method = main_method.unwrap();

    let mut stack = vm.new_stack();
    let main_result = vm.invoke(&mut stack, &main_method, None, vec![]);
    assert!(main_result.is_ok());
    assert!(main_result.unwrap().is_none());
}
