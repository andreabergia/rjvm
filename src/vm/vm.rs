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

#[derive(Debug, Default)]
pub struct Stack {
    frames: Vec<Rc<RefCell<CallFrame>>>,
}

impl Stack {
    pub fn new() -> Stack {
        Default::default()
    }

    pub fn add_frame(
        &mut self,
        class_and_method: ClassAndMethod,
        receiver: Option<ObjectRef>,
        args: Vec<Value>,
    ) -> Result<Rc<RefCell<CallFrame>>, VmError> {
        if class_and_method.method.flags.contains(MethodFlags::STATIC) {
            if receiver.is_some() {
                return Err(VmError::ValidationException);
            }
        } else if receiver.is_none() {
            return Err(VmError::NullPointerException);
        }

        let mut locals: Vec<Value> = receiver
            .map(Value::Object)
            .into_iter()
            .chain(args.into_iter())
            .collect();

        while locals.len() < class_and_method.method.code.max_locals.into_usize_safe() {
            locals.push(Value::Uninitialized);
        }

        let new_frame = CallFrame::new(class_and_method, locals);
        let new_frame = Rc::new(RefCell::new(new_frame));
        self.frames.push(Rc::clone(&new_frame));
        Ok(new_frame)
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
            self.debug_print_status(instruction);
            self.pc += 1;

            match instruction.op_code {
                OpCode::Aload_0 => {
                    let local = self.locals.get(0).ok_or(VmError::ValidationException)?;
                    self.stack.push(local.clone());
                }

                OpCode::New => {
                    let constant_index = instruction.argument_u16(0)?;
                    let new_object_class_name =
                        self.get_constant_class_reference(constant_index)?;
                    let new_object = vm.new_object(new_object_class_name)?;
                    self.stack.push(Value::Object(new_object));
                }

                _ => {
                    warn!("Unsupported op code: {}", instruction.op_code);
                    return Err(VmError::NotImplemented);
                }
            }
        }

        Err(VmError::NullPointerException)
    }

    fn get_constant(&self, constant_index: u16) -> Result<&ConstantPoolEntry, VmError> {
        self.class_and_method
            .class
            .constants
            .get(constant_index)
            .map_err(|_| VmError::ValidationException)
    }

    fn get_constant_class_reference(&self, constant_index: u16) -> Result<&str, VmError> {
        let constant = self.get_constant(constant_index)?;
        if let &ConstantPoolEntry::ClassReference(constant_index) = constant {
            self.get_constant_utf8(constant_index)
        } else {
            Err(VmError::ValidationException)
        }
    }

    fn get_constant_utf8(&self, constant_index: u16) -> Result<&str, VmError> {
        let constant = self.get_constant(constant_index)?;
        if let ConstantPoolEntry::Utf8(string) = constant {
            Ok(string)
        } else {
            Err(VmError::ValidationException)
        }
    }

    fn debug_start_execution(&self) {
        debug!(
            "starting execution of method {}::{} - locals are {:?}",
            self.class_and_method.class.name, self.class_and_method.method.name, self.locals
        )
    }

    fn debug_print_status(&self, instruction: &Instruction) {
        debug!(
            "FRAME STATUS: pc: {}, next instruction: {}",
            self.pc, instruction
        );
        debug!("  stack:");
        for stack_entry in self.stack.iter() {
            debug!("  - {:?}", stack_entry);
        }
        debug!("  locals:");
        for local_variable in self.locals.iter() {
            debug!("  - {:?}", local_variable);
        }
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
        let frame = stack.add_frame(class_and_method, object, args)?;
        let result = frame.borrow_mut().execute(self);
        result
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
