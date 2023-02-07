use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use log::{debug, warn};

use crate::reader::class_file::ClassFile;
use crate::reader::constant_pool::ConstantPoolEntry;
use crate::reader::instruction::Instruction;
use crate::reader::method_flags::MethodFlags;
use crate::reader::opcodes::OpCode;
use crate::utils::type_conversion::ToUsizeSafe;
use crate::vm::class_and_method::ClassAndMethod;
use crate::vm::value::Value;

#[derive(Debug, Default)]
pub struct Vm {
    classes: HashMap<String, ClassFile>,
}

#[derive(Debug)]
pub enum VmError {
    NullPointerException,
    ValidationException,
    NotImplemented,
}

#[derive(Debug, Default)]
pub struct Stack<'a> {
    frames: Vec<Rc<RefCell<CallFrame<'a>>>>,
}

impl<'a> Stack<'a> {
    pub fn add_frame(
        &mut self,
        class_and_method: &'a ClassAndMethod,
        receiver: Option<&'a Value>,
        args: Vec<&'a Value>,
    ) -> Rc<RefCell<CallFrame<'a>>> {
        // TODO: verify local size with static method data
        let locals = receiver.into_iter().chain(args.into_iter()).collect();
        let new_frame = Rc::new(RefCell::new(CallFrame::new(class_and_method, locals)));
        self.frames.push(Rc::clone(&new_frame));
        new_frame
    }
}

#[derive(Debug)]
pub struct CallFrame<'a> {
    class_and_method: &'a ClassAndMethod<'a>,
    pc: usize,
    locals: Vec<&'a Value>,
    stack: Vec<&'a Value>,
}

impl<'a> CallFrame<'a> {
    fn new(class_and_method: &'a ClassAndMethod, locals: Vec<&'a Value>) -> CallFrame<'a> {
        CallFrame {
            class_and_method,
            pc: 0,
            locals,
            stack: Vec::with_capacity(class_and_method.method.code.max_stack.into_usize_safe()),
        }
    }

    pub fn execute(&mut self) -> Result<Option<&'a Value>, VmError> {
        self.debug_start_execution();

        let code = &self.class_and_method.method.code.code;
        for instruction in code {
            self.debug_print_stack(instruction);
            self.pc += 1;

            match instruction.op_code {
                OpCode::Aload_0 => {
                    let local = *self.locals.get(0).ok_or(VmError::ValidationException)?;
                    self.stack.push(local);
                }

                OpCode::New => {
                    let index_byte_1 = instruction.argument(0)?;
                    let index_byte_2 = instruction.argument(1)?;
                    let constant_index = ((index_byte_1 as u16) << 8) | index_byte_2 as u16;
                    let constant = self.get_constant(constant_index)?;
                    if let &ConstantPoolEntry::ClassReference(constant_index) = constant {
                        let constant = self.get_constant(constant_index)?;
                        if let ConstantPoolEntry::Utf8(new_object_class_name) = constant {
                            warn!("Should create instance of object {}", new_object_class_name);
                            return Err(VmError::NotImplemented);
                        } else {
                            return Err(VmError::ValidationException);
                        }
                    } else {
                        return Err(VmError::ValidationException);
                    }
                }

                _ => {
                    warn!("Unsupported op code: {}", instruction.op_code);
                    return Err(VmError::NotImplemented);
                }
            }
        }

        Err(VmError::NullPointerException)
    }

    fn get_constant(&self, index: u16) -> Result<&ConstantPoolEntry, VmError> {
        self.class_and_method
            .class
            .constants
            .get(index)
            .map_err(|_| VmError::ValidationException)
    }

    fn debug_start_execution(&self) {
        debug!(
            "starting execution of method {}::{} - locals are {:?}",
            self.class_and_method.class.name, self.class_and_method.method.name, self.locals
        )
    }

    fn debug_print_stack(&self, instruction: &Instruction) {
        debug!(
            "- pc: {}, stack: {:?}, next instruction: {}",
            self.pc, self.stack, instruction
        );
    }
}

impl Vm {
    pub fn new_stack(&self) -> Stack {
        Default::default()
    }

    pub fn invoke<'a>(
        &self,
        stack: &'a mut Stack<'a>,
        class_and_method: &'a ClassAndMethod,
        object: Option<&'a Value>,
        args: Vec<&'a Value>,
    ) -> Result<Option<&'a Value>, VmError> {
        if object.is_none() {
            if !class_and_method.method.flags.contains(MethodFlags::STATIC) {
                return Err(VmError::NullPointerException);
            }
            println!("invoking static");

            let mut frame = stack.add_frame(class_and_method, object, args);
            let result = frame.borrow_mut().execute();
            result
        } else {
            Ok(None)
        }
    }
}

impl Vm {
    pub fn load_class(&mut self, class_file: ClassFile) {
        self.classes.insert(class_file.name.clone(), class_file);
    }

    pub fn find_class(&self, class_name: &str) -> Option<&ClassFile> {
        self.classes.get(class_name)
    }
}

impl Vm {
    pub fn new() -> Vm {
        Default::default()
    }
}
