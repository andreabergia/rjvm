extern crate rjvm;

use rjvm::vm::class_and_method::ClassAndMethod;

mod utils;

#[test_log::test]
fn can_execute_real_code() {
    let class_simple_main = utils::read_class_from_file("rjvm/SimpleMain");
    let class_generator = utils::read_class_from_file("rjvm/SimpleMain$Generator");
    let class_java_lang_object = utils::read_class_from_file("jre-8-rt/java/lang/Object");

    let mut vm = rjvm::vm::Vm::new();
    vm.load_class(class_simple_main);
    vm.load_class(class_generator);
    vm.load_class(class_java_lang_object);

    let main_method = vm.find_class("rjvm/SimpleMain").and_then(|class| {
        class
            .find_method("main", "([Ljava/lang/String;)V")
            .map(|method| ClassAndMethod { class, method })
    });
    assert!(main_method.is_some());
    let main_method = main_method.unwrap();

    let mut stack = vm.new_stack();
    let main_result = vm.invoke(&mut stack, main_method, None, vec![]);
    print!("result: {main_result:?}");
    assert!(main_result.is_ok());
    assert!(main_result.unwrap().is_none());
}
