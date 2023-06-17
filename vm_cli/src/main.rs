use clap::Parser;

use rjvm_vm::call_stack::CallStack;
use rjvm_vm::{
    class_and_method::ClassAndMethod,
    exceptions::MethodCallFailed,
    vm::{Vm, DEFAULT_MAX_MEMORY},
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

    /// Java program arguments
    java_program_arguments: Vec<String>,
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
    let mut vm = Vm::new(DEFAULT_MAX_MEMORY);
    append_classpath(&mut vm, &args)?;

    let (call_stack, main_method) = resolve_class_and_main_method(&mut vm, &args)?;

    // TODO: args
    let main_result = vm
        .invoke(call_stack, main_method, None, vec![])
        .map_err(|v| format!("execution error: {:?}", v))?;

    match main_result {
        None => Ok(0),
        Some(v) => Err(format!(
            "<main> method should be void, but returned the value: {v:?}",
        )),
    }
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
