extern crate rjvm;

use rjvm::{
    reader::class_file::ClassFile,
    reader::class_file_field::ClassFileField,
    reader::class_file_method::ClassFileMethod,
    reader::field_flags::FieldFlags,
    reader::method_flags::MethodFlags,
    reader::{class_access_flags::ClassAccessFlags, class_file_version::ClassFileVersion},
};

mod utils;

#[test_log::test]
fn can_read_simple_main() {
    let class_main = utils::read_class_from_file("SimpleMain");
    let class_generator = utils::read_class_from_file("SimpleMain$Generator");

    let mut vm = rjvm::vm::Vm::new();
    vm.load_class(class_main);
    vm.load_class(class_generator);

    let main = vm
        .find_class("rjvm/SimpleMain")
        .map(|class| class.find_method("main", "([Ljava/lang/String;)V"));
    assert!(main.is_some());
}
