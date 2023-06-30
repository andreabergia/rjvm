extern crate rjvm_reader;

use rjvm_reader::{
    class_file::ClassFile,
    exception_table::{ExceptionTable, ExceptionTableEntry},
    method_flags::MethodFlags,
    program_counter::ProgramCounter,
};
use utils::read_class_from_bytes;

use crate::{assertions::check_method, utils};

#[test_log::test]
fn can_read_class_with_exception_handler_and_throws() {
    let class = read_class_from_bytes(include_bytes!("../resources/rjvm/ExceptionsHandlers.class"));
    assert_eq!("rjvm/ExceptionsHandlers", class.name);

    check_methods(&class);
}

fn check_methods(class: &ClassFile) {
    assert_eq!(4, class.methods.len());

    check_method(&class.methods[0], MethodFlags::empty(), "<init>", "()V");
    check_method(&class.methods[1], MethodFlags::empty(), "foo", "()V");

    check_method(&class.methods[2], MethodFlags::empty(), "bar", "()V");
    assert_eq!(
        vec![
            "java/lang/IllegalArgumentException".to_string(),
            "java/lang/IllegalStateException".to_string()
        ],
        class.methods[2].thrown_exceptions
    );

    check_method(&class.methods[3], MethodFlags::empty(), "test", "()V");
    assert_eq!(
        ExceptionTable::new(vec![
            ExceptionTableEntry {
                range: ProgramCounter(0)..ProgramCounter(4),
                handler_pc: ProgramCounter(11),
                catch_class: None
            },
            ExceptionTableEntry {
                range: ProgramCounter(18)..ProgramCounter(22),
                handler_pc: ProgramCounter(25),
                catch_class: Some("java/lang/IllegalStateException".to_string())
            }
        ]),
        class.methods[3].code.as_ref().unwrap().exception_table
    );
}
