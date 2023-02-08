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
use crate::vm::value::{ObjectRef, ObjectValue, Value};
use crate::vm::vm_error::VmError;

#[derive(Debug)]
pub struct Stack {
    frames: Vec<Rc<RefCell<CallFrame>>>,
}

impl Stack {
    pub fn new() -> Stack {
        Stack { frames: Vec::new() }
    }

    pub fn add_frame(
        &mut self,
        class_and_method: ClassAndMethod,
        receiver: Option<ObjectRef>,
        args: Vec<Value>,
    ) -> Rc<RefCell<CallFrame>> {
        // TODO: verify local size with static method data
        let locals = receiver
            .map(Value::Object)
            .into_iter()
            .chain(args.into_iter())
            .collect();
        let new_frame = CallFrame::new(class_and_method, locals);
        let new_frame = Rc::new(RefCell::new(new_frame));
        self.frames.push(Rc::clone(&new_frame));
        new_frame
    }
}

#[derive(Debug)]
pub struct CallFrame {
    class_and_method: ClassAndMethod,
    pc: usize,
    locals: Vec<Value>,
    stack: Vec<Value>,
}

impl CallFrame {
    fn new(class_and_method: ClassAndMethod, locals: Vec<Value>) -> CallFrame {
        let max_stack_size = class_and_method.method.code.max_stack.into_usize_safe();
        CallFrame {
            class_and_method,
            pc: 0,
            locals,
            stack: Vec::with_capacity(max_stack_size),
        }
    }

    pub fn execute(&mut self, vm: &mut Vm) -> Result<Option<Value>, VmError> {
        self.debug_start_execution();

        let code = &self.class_and_method.method.code.code;
        for instruction in code {
            self.debug_print_stack(instruction);
            self.pc += 1;

            match instruction.op_code {
                OpCode::Aload_0 => {
                    let local = self.locals.get(0).ok_or(VmError::ValidationException)?;
                    self.stack.push(local.clone());
                }

                OpCode::New => {
                    let index_byte_1 = instruction.argument(0)?;
                    let index_byte_2 = instruction.argument(1)?;
                    let constant_index = ((index_byte_1 as u16) << 8) | index_byte_2 as u16;
                    let constant = self.get_constant(constant_index)?;
                    if let &ConstantPoolEntry::ClassReference(constant_index) = constant {
                        let constant = self.get_constant(constant_index)?;
                        if let ConstantPoolEntry::Utf8(new_object_class_name) = constant {
                            let new_object = vm.new_object(new_object_class_name)?;
                            self.stack.push(Value::Object(new_object));
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

#[derive(Debug, Default)]
pub struct Vm {
    classes: HashMap<String, Rc<ClassFile>>,
    heap: Vec<ObjectRef>,
}

impl Vm {
    pub fn new() -> Vm {
        Vm {
            classes: Default::default(),
            heap: Vec::new(),
        }
    }

    pub fn load_class(&mut self, class_file: ClassFile) {
        let class_file = Rc::new(class_file);
        self.classes.insert(class_file.name.clone(), class_file);
    }

    pub fn find_class(&self, class_name: &str) -> Option<Rc<ClassFile>> {
        self.classes.get(class_name).map(Rc::clone)
    }

    // TODO: do we need it?
    pub fn new_stack(&self) -> Stack {
        Stack::new()
    }

    pub fn invoke(
        &mut self,
        stack: &mut Stack,
        class_and_method: ClassAndMethod,
        object: Option<ObjectRef>,
        args: Vec<Value>,
    ) -> Result<Option<Value>, VmError> {
        if object.is_none() {
            if !class_and_method.method.flags.contains(MethodFlags::STATIC) {
                return Err(VmError::NullPointerException);
            }
            println!("invoking static");

            let frame = stack.add_frame(class_and_method, object, args);
            let result = frame.borrow_mut().execute(self);
            Ok(None)
        } else {
            Ok(None)
        }
    }

    pub fn new_object(&mut self, class_name: &str) -> Result<ObjectRef, VmError> {
        debug!("allocating new instance of {}", class_name);

        let instance = self
            .classes
            .get(class_name)
            .ok_or(VmError::ClassNotFoundException(class_name.to_string()))
            .map(|class| ObjectValue {
                class: Rc::clone(class),
                fields: class.fields.iter().map(|_| Value::Uninitialized).collect(),
            })?;
        let instance = Rc::new(RefCell::new(instance));
        self.heap.push(instance.clone());
        Ok(instance)
    }
}
