use clap::Parser;

use rjvm_vm::{
    array::Array,
    array_entry_type::ArrayEntryType,
    call_stack::CallStack,
    class_and_method::ClassAndMethod,
    exceptions::MethodCallFailed,
    java_objects_creation::new_java_lang_string_object,
    value::Value,
    vm::{Vm, DEFAULT_MAX_MEMORY_MB_STR, ONE_MEGABYTE},
    vm_error::VmError,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Class path. Use colon (:) as separator for entries
    #[arg(short, long)]
    classpath: Option<String>,

    /// Class name to execute
    class_name: String,

    /// Maximum memory to use in MB
    #[arg(short, long, default_value = DEFAULT_MAX_MEMORY_MB_STR)]
    maximum_mb_of_memory: usize,

    /// Java program arguments
    java_program_arguments: Vec<String>,
}

fn main() {
    let args = Args::parse();
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let result = run(args);
    match result {
        Ok(exit_code) => std::process::exit(exit_code),
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(-1);
        }
    }
}

fn append_classpath(vm: &mut Vm, args: &Args) -> Result<(), String> {
    if let Some(classpath) = &args.classpath {
        vm.append_class_path(classpath)
            .map_err(|err| err.to_string())?;
    }
    Ok(())
}

fn resolve_class_and_main_method<'a>(
    vm: &mut Vm<'a>,
    args: &Args,
) -> Result<(&'a mut CallStack<'a>, ClassAndMethod<'a>), String> {
    let call_stack = vm.allocate_call_stack();
    let main_method = vm
        .resolve_class_method(
            call_stack,
            &args.class_name,
            "main",
            "([Ljava/lang/String;)V",
        )
        .map_err(|v| match v {
            MethodCallFailed::InternalError(VmError::ClassNotFoundException(name)) => {
                format!("class not found: {name}")
            }
            MethodCallFailed::InternalError(VmError::MethodNotFoundException(..)) => {
                "class does not contain a valid <main> method".to_string()
            }
            _ => format!("unexpected error: {:?}", v),
        })?;
    Ok((call_stack, main_method))
}

fn run(args: Args) -> Result<i32, String> {
    let mut vm = Vm::new(args.maximum_mb_of_memory * ONE_MEGABYTE);
    append_classpath(&mut vm, &args)?;

    let (call_stack, main_method) = resolve_class_and_main_method(&mut vm, &args)?;

    let main_args = allocate_java_args(&mut vm, call_stack, &args.java_program_arguments)
        .map_err(|err| format!("{err:?}"))?;
    let main_result = vm
        .invoke(call_stack, main_method, None, vec![main_args])
        .map_err(|v| format!("execution error: {:?}", v))?;

    match main_result {
        None => Ok(0),
        Some(v) => Err(format!(
            "<main> method should be void, but returned the value: {v:?}",
        )),
    }
}

fn allocate_java_args<'a>(
    vm: &mut Vm<'a>,
    call_stack: &mut CallStack<'a>,
    command_line_args: &[String],
) -> Result<Value<'a>, MethodCallFailed<'a>> {
    let class_id_java_lang_string = vm.get_or_resolve_class(call_stack, "java/lang/String")?.id;

    let strings: Result<Vec<Value<'a>>, MethodCallFailed<'a>> = command_line_args
        .iter()
        .map(|s| new_java_lang_string_object(vm, call_stack, s).map(Value::Object))
        .collect();

    let strings = strings?;
    let array = vm.new_array(
        ArrayEntryType::Object(class_id_java_lang_string),
        strings.len(),
    );

    for (index, string) in strings.into_iter().enumerate() {
        array.set_element(index, string)?;
    }
    Ok(Value::Object(array))
}
