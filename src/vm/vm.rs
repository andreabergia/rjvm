use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use log::{debug, info, warn};

use crate::reader::field_type::FieldType::Base;
use crate::reader::field_type::{BaseType, FieldType};
use crate::vm::value::Value::Object;
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

        if class_and_method.is_native() {
            return Err(VmError::NotImplemented);
        };

        let code = &class_and_method.method.code.as_ref().unwrap();

        let mut locals: Vec<Value> = receiver
            .map(Object)
            .into_iter()
            .chain(args.into_iter())
            .collect();

        while locals.len() < code.max_locals.into_usize_safe() {
            locals.push(Value::Uninitialized);
        }

        let new_frame = CallFrame::new(class_and_method, locals);
        let new_frame = Rc::new(RefCell::new(new_frame));
        self.frames.push(Rc::clone(&new_frame));
        Ok(new_frame)
    }

    pub fn pop_frame(&mut self) -> Result<(), VmError> {
        self.frames
            .pop()
            .map(|_| ())
            .ok_or(VmError::ValidationException)
    }
}

#[derive(Debug)]
struct MethodReference<'a> {
    class_name: &'a str,
    method_name: &'a str,
    type_descriptor: &'a str,
}

#[derive(Debug)]
struct FieldReference<'a> {
    class_name: &'a str,
    field_name: &'a str,
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
        let max_stack_size = class_and_method
            .method
            .code
            .as_ref()
            .expect("method is not native")
            .max_stack
            .into_usize_safe();
        CallFrame {
            class_and_method,
            pc: 0,
            locals,
            stack: Vec::with_capacity(max_stack_size),
        }
    }

    pub fn execute(&mut self, vm: &mut Vm, stack: &mut Stack) -> Result<Option<Value>, VmError> {
        self.debug_start_execution();

        let code = &self
            .class_and_method
            .method
            .code
            .as_ref()
            .expect("method is not native")
            .code;
        for instruction in code {
            self.debug_print_status(instruction);
            self.pc += 1;

            match instruction.op_code {
                OpCode::Aload => {
                    let index = instruction.argument(0)?.into_usize_safe();
                    let local = self.locals.get(index).ok_or(VmError::ValidationException)?;
                    self.stack.push(local.clone());
                }
                OpCode::Aload_0 => {
                    let local = self.locals.get(0).ok_or(VmError::ValidationException)?;
                    self.stack.push(local.clone());
                }
                OpCode::Aload_1 => {
                    let local = self.locals.get(1).ok_or(VmError::ValidationException)?;
                    self.stack.push(local.clone());
                }
                OpCode::Aload_2 => {
                    let local = self.locals.get(2).ok_or(VmError::ValidationException)?;
                    self.stack.push(local.clone());
                }
                OpCode::Aload_3 => {
                    let local = self.locals.get(3).ok_or(VmError::ValidationException)?;
                    self.stack.push(local.clone());
                }

                OpCode::Astore => {
                    let index = instruction.argument(0)?.into_usize_safe();
                    let value = self.stack.pop().ok_or(VmError::ValidationException)?;
                    // TODO: validate is object
                    self.locals[index] = value;
                }
                OpCode::Astore_0 => {
                    let value = self.stack.pop().ok_or(VmError::ValidationException)?;
                    self.locals[0] = value;
                }
                OpCode::Astore_1 => {
                    let value = self.stack.pop().ok_or(VmError::ValidationException)?;
                    self.locals[1] = value;
                }
                OpCode::Astore_2 => {
                    let value = self.stack.pop().ok_or(VmError::ValidationException)?;
                    self.locals[2] = value;
                }
                OpCode::Astore_3 => {
                    let value = self.stack.pop().ok_or(VmError::ValidationException)?;
                    self.locals[3] = value;
                }

                OpCode::New => {
                    let constant_index = instruction.arguments_u16(0)?;
                    let new_object_class_name =
                        self.get_constant_class_reference(constant_index)?;
                    let new_object = vm.new_object(new_object_class_name)?;
                    self.stack.push(Object(new_object));
                }

                OpCode::Dup => {
                    let stack_head = self.stack.last().ok_or(VmError::ValidationException)?;
                    self.stack.push(stack_head.clone());
                }
                OpCode::Pop => {
                    self.stack.pop().ok_or(VmError::ValidationException)?;
                }

                OpCode::Iconst_m1 => self.stack.push(Value::Int(-1)),
                OpCode::Iconst_0 => self.stack.push(Value::Int(0)),
                OpCode::Iconst_1 => self.stack.push(Value::Int(1)),
                OpCode::Iconst_2 => self.stack.push(Value::Int(2)),
                OpCode::Iconst_3 => self.stack.push(Value::Int(3)),
                OpCode::Iconst_4 => self.stack.push(Value::Int(4)),
                OpCode::Iconst_5 => self.stack.push(Value::Int(5)),

                OpCode::Invokespecial => {
                    let class_and_method = self.get_method_to_invoke(vm, instruction)?;
                    let (receiver, params, new_stack_len) =
                        self.get_method_receiver_and_params(&class_and_method)?;
                    self.stack.truncate(new_stack_len);

                    let method_return_type = class_and_method.return_type();
                    let result = vm.invoke(stack, class_and_method, receiver, params)?;

                    Self::validate_type(method_return_type, &result)?;
                    if let Some(value) = result {
                        self.stack.push(value);
                    }
                }

                OpCode::Invokestatic => {
                    let class_and_method = self.get_method_to_invoke(vm, instruction)?;
                    let (receiver, params, new_stack_len) =
                        self.get_method_receiver_and_params(&class_and_method)?;
                    self.stack.truncate(new_stack_len);

                    let method_return_type = class_and_method.return_type();
                    let result = vm.invoke(stack, class_and_method, receiver, params)?;

                    Self::validate_type(method_return_type, &result)?;
                    if let Some(value) = result {
                        self.stack.push(value);
                    }
                }

                OpCode::Invokevirtual => {
                    // TODO: should actually go through superclasses to find the correct method
                    let class_and_method = self.get_method_to_invoke(vm, instruction)?;
                    let (receiver, params, new_stack_len) =
                        self.get_method_receiver_and_params(&class_and_method)?;
                    self.stack.truncate(new_stack_len);

                    let method_return_type = class_and_method.return_type();
                    let result = vm.invoke(stack, class_and_method, receiver, params)?;

                    Self::validate_type(method_return_type, &result)?;
                    if let Some(value) = result {
                        self.stack.push(value);
                    }
                }

                OpCode::Return => {
                    if !self.class_and_method.is_void() {
                        return Err(VmError::ValidationException);
                    }
                    self.debug_done_execution(None);
                    return Ok(None);
                }
                OpCode::Ireturn => {
                    if !self.class_and_method.returns(Base(BaseType::Int)) {
                        return Err(VmError::ValidationException);
                    }
                    let result = self.stack.pop().ok_or(VmError::ValidationException)?;
                    self.debug_done_execution(Some(&result));
                    return Ok(Some(result));
                }

                OpCode::Iload => {
                    let index = instruction.argument(0)?.into_usize_safe();
                    self.stack.push(self.get_local_int(index)?.clone());
                }
                OpCode::Iload_0 => {
                    self.stack.push(self.get_local_int(0)?.clone());
                }
                OpCode::Iload_1 => {
                    self.stack.push(self.get_local_int(1)?.clone());
                }
                OpCode::Iload_2 => {
                    self.stack.push(self.get_local_int(2)?.clone());
                }
                OpCode::Iload_3 => {
                    self.stack.push(self.get_local_int(3)?.clone());
                }

                OpCode::Putfield => {
                    let field_index = instruction.arguments_u16(0)?;
                    let field_reference = self.get_constant_field_reference(field_index)?;

                    // TODO: validate class? How do super classes work?
                    let (index, field) = self
                        .class_and_method
                        .class
                        .get_field(field_reference.field_name)?;

                    let value = self.stack.pop().ok_or(VmError::ValidationException)?;
                    Self::validate_type(Some(field.type_descriptor.clone()), &Some(value.clone()))?;
                    let object = self.stack.pop().ok_or(VmError::ValidationException)?;
                    if let Object(object_ref) = object {
                        object_ref.borrow_mut().fields[index] = value;
                    } else {
                        return Err(VmError::ValidationException);
                    }
                }

                OpCode::Getfield => {
                    let field_index = instruction.arguments_u16(0)?;
                    let field_reference = self.get_constant_field_reference(field_index)?;

                    // TODO: validate class? How do super classes work?
                    let (index, field) = self
                        .class_and_method
                        .class
                        .get_field(field_reference.field_name)?;

                    let object = self.stack.pop().ok_or(VmError::ValidationException)?;
                    if let Object(object_ref) = object {
                        let object_ref: &RefCell<ObjectValue> = object_ref.borrow();
                        let field_value = object_ref.borrow().fields[index].clone();
                        Self::validate_type(
                            Some(field.type_descriptor.clone()),
                            &Some(field_value.clone()),
                        )?;
                        self.stack.push(field_value);
                    } else {
                        return Err(VmError::ValidationException);
                    }
                }

                OpCode::Iadd => {
                    let i1 = Self::pop_int(&mut self.stack)?;
                    let i2 = Self::pop_int(&mut self.stack)?;
                    self.stack.push(Value::Int(i1 + i2));
                }

                _ => {
                    warn!("Unsupported op code: {}", instruction.op_code);
                    return Err(VmError::NotImplemented);
                }
            }
        }

        Err(VmError::NullPointerException)
    }

    fn pop_int(stack: &mut Vec<Value>) -> Result<i32, VmError> {
        let value = stack.pop().ok_or(VmError::ValidationException)?;
        Self::validate_type(Some(FieldType::Base(BaseType::Int)), &Some(value.clone()))?;
        match value {
            Value::Int(int) => Ok(int),
            _ => Err(VmError::ValidationException),
        }
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

    fn get_constant_field_reference(&self, constant_index: u16) -> Result<FieldReference, VmError> {
        let constant = self.get_constant(constant_index)?;
        if let &ConstantPoolEntry::FieldReference(
            class_name_index,
            name_and_type_descriptor_index,
        ) = constant
        {
            let class_name = self.get_constant_class_reference(class_name_index)?;
            let constant = self.get_constant(name_and_type_descriptor_index)?;
            if let &ConstantPoolEntry::NameAndTypeDescriptor(name_index, type_descriptor_index) =
                constant
            {
                let field_name = self.get_constant_utf8(name_index)?;
                let type_descriptor = self.get_constant_utf8(type_descriptor_index)?;
                return Ok(FieldReference {
                    class_name,
                    field_name,
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

    fn get_method_to_invoke(
        &self,
        vm: &Vm,
        instruction: &Instruction,
    ) -> Result<ClassAndMethod, VmError> {
        let constant_index = instruction.arguments_u16(0)?;
        let method_reference = self.get_constant_method_reference(constant_index)?;

        let class = vm.get_class(method_reference.class_name)?;
        let method = class.get_method(
            method_reference.method_name,
            method_reference.type_descriptor,
        )?;
        Ok(ClassAndMethod { class, method })
    }

    fn get_method_receiver_and_params(
        &self,
        class_and_method: &ClassAndMethod,
    ) -> Result<(Option<ObjectRef>, Vec<Value>, usize), VmError> {
        let cur_stack_len = self.stack.len();
        let receiver_count = if class_and_method.is_static() { 0 } else { 1 };
        let num_params = class_and_method.num_arguments();
        if cur_stack_len < (receiver_count + num_params) {
            return Err(VmError::ValidationException);
        }

        let receiver = if class_and_method.is_static() {
            None
        } else {
            Some(self.get_object_from_stack(
                cur_stack_len - num_params - receiver_count,
                &class_and_method.class,
            )?)
        };
        let params = Vec::from(&self.stack[cur_stack_len - num_params..cur_stack_len]);
        Ok((
            receiver,
            params,
            cur_stack_len - num_params - receiver_count,
        ))
    }

    fn get_object_from_stack(
        &self,
        index: usize,
        _expected_class: &ClassFile,
    ) -> Result<ObjectRef, VmError> {
        let receiver = self.stack.get(index).ok_or(VmError::ValidationException)?;
        match receiver {
            Object(object) => {
                // TODO: here we should check "instanceof" the expected class of a subclass
                Ok(object.clone())
            }
            _ => Err(VmError::ValidationException),
        }
    }

    fn validate_type(
        expected_type: Option<FieldType>,
        value: &Option<Value>,
    ) -> Result<(), VmError> {
        match expected_type {
            None => match value {
                None => Ok(()),
                Some(_) => Err(VmError::ValidationException),
            },
            Some(expected_type) => match value {
                None => Err(VmError::ValidationException),
                Some(value) => {
                    if value.matches_type(expected_type) {
                        Ok(())
                    } else {
                        Err(VmError::ValidationException)
                    }
                }
            },
        }
    }

    fn get_local_int(&self, index: usize) -> Result<&Value, VmError> {
        // TODO: short, char, byte should (probably?) to be modelled as int
        let variable = self.locals.get(index).ok_or(VmError::ValidationException)?;
        Self::validate_type(
            Some(FieldType::Base(BaseType::Int)),
            &Some(variable.clone()),
        )?;
        Ok(variable)
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

    fn debug_done_execution(&self, result: Option<&Value>) {
        debug!(
            "completed execution of method {}::{} - result is {:?}",
            self.class_and_method.class.name, self.class_and_method.method.name, result
        )
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
        if class_and_method.method.is_native() {
            if class_and_method.class.name == "rjvm/SimpleMain"
                && class_and_method.method.name == "tempPrint"
            {
                let arg = args.get(0).ok_or(VmError::ValidationException)?;
                info!("TEMP implementation of native method: printing value {arg:?}");
                return Ok(None);
            } else {
                return Err(VmError::NotImplemented);
            }
        }

        let frame = stack.add_frame(class_and_method, object, args)?;
        let result = frame.borrow_mut().execute(self, stack);
        stack.pop_frame()?;
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
