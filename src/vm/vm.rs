use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use log::{debug, warn};

use crate::{
    reader::{
        class_file::ClassFile, constant_pool::ConstantPoolEntry, instruction::Instruction,
        method_flags::MethodFlags, opcodes::OpCode,
    },
    utils::type_conversion::ToUsizeSafe,
    vm::{
        class_and_method::ClassAndMethod,
        value::{ObjectRef, ObjectValue, Value},
        vm_error::VmError,
    },
};

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
struct MethodReference<'a> {
    class_name: &'a str,
    method_name: &'a str,
    type_descriptor: &'a str,
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

                OpCode::Dup => {
                    let stack_head = self.stack.last().ok_or(VmError::ValidationException)?;
                    self.stack.push(stack_head.clone());
                }

                OpCode::Iconst_m1 => self.stack.push(Value::Int(-1)),
                OpCode::Iconst_0 => self.stack.push(Value::Int(0)),
                OpCode::Iconst_1 => self.stack.push(Value::Int(1)),
                OpCode::Iconst_2 => self.stack.push(Value::Int(2)),
                OpCode::Iconst_3 => self.stack.push(Value::Int(3)),
                OpCode::Iconst_4 => self.stack.push(Value::Int(4)),
                OpCode::Iconst_5 => self.stack.push(Value::Int(5)),

                OpCode::Invokespecial => {
                    let constant_index = instruction.argument_u16(0)?;
                    let method_reference = self.get_constant_method_reference(constant_index)?;

                    let class = vm.get_class(method_reference.class_name)?;
                    let method = class.get_method(
                        method_reference.method_name,
                        method_reference.type_descriptor,
                    )?;

                    warn!(
                        "TODO: should invoke method {}.{} of type {}",
                        class.name, method.name, method.parsed_type_descriptor,
                    );
                    return Err(VmError::NotImplemented);
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

    fn get_constant_method_reference(
        &self,
        constant_index: u16,
    ) -> Result<MethodReference, VmError> {
        let constant = self.get_constant(constant_index)?;
        if let &ConstantPoolEntry::MethodReference(
            class_name_index,
            name_and_type_descriptor_index,
        ) = constant
        {
            let class_name = self.get_constant_class_reference(class_name_index)?;
            let constant = self.get_constant(name_and_type_descriptor_index)?;
            if let &ConstantPoolEntry::NameAndTypeDescriptor(name_index, type_descriptor_index) =
                constant
            {
                let method_name = self.get_constant_utf8(name_index)?;
                let type_descriptor = self.get_constant_utf8(type_descriptor_index)?;
                return Ok(MethodReference {
                    class_name,
                    method_name,
                    type_descriptor,
                });
            }
        }
        Err(VmError::ValidationException)
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

    pub fn get_class(&self, class_name: &str) -> Result<Rc<ClassFile>, VmError> {
        self.find_class(class_name)
            .ok_or(VmError::ClassNotFoundException(class_name.to_string()))
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

        let instance = self.get_class(class_name).map(|class| ObjectValue {
            class: Rc::clone(&class),
            fields: class.fields.iter().map(|_| Value::Uninitialized).collect(),
        })?;
        let instance = Rc::new(RefCell::new(instance));
        self.heap.push(instance.clone());
        Ok(instance)
    }
}
