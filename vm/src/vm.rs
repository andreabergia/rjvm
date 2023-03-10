use std::cell::RefCell;
use std::rc::Rc;

use log::{debug, info, warn};

use rjvm_reader::{
    class_file::ClassFile, class_file_field::ClassFileField, class_file_method::ClassFileMethod,
    constant_pool::ConstantPoolEntry, field_type::BaseType, field_type::FieldType,
    field_type::FieldType::Base, instruction::Instruction, method_flags::MethodFlags,
    opcodes::OpCode,
};
use rjvm_utils::type_conversion::ToUsizeSafe;

use crate::class::ClassRef;
use crate::class_allocator::{ClassAllocator, ClassResolver};
use crate::gc::ObjectAllocator;
use crate::value::ObjectRef;
use crate::{
    class::Class, class_and_method::ClassAndMethod, class_loader::ClassLoader, value::Value,
    value::Value::Object, vm_error::VmError,
};

#[derive(Debug, Default)]
pub struct Stack<'a> {
    frames: Vec<Rc<RefCell<CallFrame<'a>>>>,
}

impl<'a> Stack<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_frame(
        &mut self,
        class_and_method: ClassAndMethod<'a>,
        receiver: Option<ObjectRef<'a>>,
        args: Vec<Value<'a>>,
    ) -> Result<Rc<RefCell<CallFrame<'a>>>, VmError> {
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

        let mut locals: Vec<Value<'a>> = receiver
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
pub struct CallFrame<'a> {
    class_and_method: ClassAndMethod<'a>,
    pc: usize,
    locals: Vec<Value<'a>>,
    stack: Vec<Value<'a>>,
    code: &'a Vec<u8>,
}

enum InvokeKind {
    Special,
    Static,
    Virtual,
}

impl<'a> CallFrame<'a> {
    fn new(class_and_method: ClassAndMethod<'a>, locals: Vec<Value<'a>>) -> Self {
        let max_stack_size = class_and_method
            .method
            .code
            .as_ref()
            .expect("method is not native")
            .max_stack
            .into_usize_safe();
        let code = &class_and_method
            .method
            .code
            .as_ref()
            .expect("method is not native")
            .code;
        CallFrame {
            class_and_method,
            pc: 0,
            locals,
            stack: Vec::with_capacity(max_stack_size),
            code,
        }
    }

    pub fn execute(
        &mut self,
        vm: &mut Vm<'a>,
        stack: &mut Stack<'a>,
    ) -> Result<Option<Value<'a>>, VmError> {
        self.debug_start_execution();

        loop {
            let instruction = Instruction::parse_instruction(self.code, self.pc)
                .map_err(|_| VmError::ValidationException)?;
            self.debug_print_status(&instruction);
            self.pc += instruction.length();

            match instruction.op_code {
                OpCode::Aconst_null => self.stack.push(Value::Null),
                OpCode::Aload => {
                    let index = instruction.argument_u8(0)?.into_usize_safe();
                    self.execute_aload(index)?;
                }
                OpCode::Aload_0 => self.execute_aload(0)?,
                OpCode::Aload_1 => self.execute_aload(1)?,
                OpCode::Aload_2 => self.execute_aload(2)?,
                OpCode::Aload_3 => self.execute_aload(3)?,

                OpCode::Astore => {
                    let index = instruction.argument_u8(0)?.into_usize_safe();
                    self.execute_astore(index)?;
                }
                OpCode::Astore_0 => self.execute_astore(0)?,
                OpCode::Astore_1 => self.execute_astore(1)?,
                OpCode::Astore_2 => self.execute_astore(2)?,
                OpCode::Astore_3 => self.execute_astore(3)?,

                OpCode::Istore => {
                    let index = instruction.argument_u8(0)?.into_usize_safe();
                    let value = self.stack.pop().ok_or(VmError::ValidationException)?;
                    // TODO: validate is int
                    self.locals[index] = value;
                }
                OpCode::Istore_0 => {
                    let value = self.stack.pop().ok_or(VmError::ValidationException)?;
                    self.locals[0] = value;
                }
                OpCode::Istore_1 => {
                    let value = self.stack.pop().ok_or(VmError::ValidationException)?;
                    self.locals[1] = value;
                }
                OpCode::Istore_2 => {
                    let value = self.stack.pop().ok_or(VmError::ValidationException)?;
                    self.locals[2] = value;
                }
                OpCode::Istore_3 => {
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
                OpCode::Bipush => {
                    let byte_value = instruction.argument_u8(0)?;
                    self.stack.push(Value::Int(byte_value as i32));
                }

                OpCode::Invokespecial => {
                    self.invoke_method(vm, stack, instruction, InvokeKind::Special)?
                }
                OpCode::Invokestatic => {
                    self.invoke_method(vm, stack, instruction, InvokeKind::Static)?
                }
                OpCode::Invokevirtual => {
                    self.invoke_method(vm, stack, instruction, InvokeKind::Virtual)?
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
                    let index = instruction.argument_u8(0)?.into_usize_safe();
                    self.stack.push(self.get_local_int(vm, index)?.clone());
                }
                OpCode::Iload_0 => {
                    self.stack.push(self.get_local_int(vm, 0)?.clone());
                }
                OpCode::Iload_1 => {
                    self.stack.push(self.get_local_int(vm, 1)?.clone());
                }
                OpCode::Iload_2 => {
                    self.stack.push(self.get_local_int(vm, 2)?.clone());
                }
                OpCode::Iload_3 => {
                    self.stack.push(self.get_local_int(vm, 3)?.clone());
                }

                OpCode::Putfield => {
                    let field_index = instruction.arguments_u16(0)?;
                    let field_reference = self.get_constant_field_reference(field_index)?;

                    // TODO: validate class? How do super classes work?
                    let (index, field) =
                        Self::get_field(self.class_and_method.class, field_reference.field_name)?;

                    let value = self.stack.pop().ok_or(VmError::ValidationException)?;
                    Self::validate_type(vm, field.type_descriptor.clone(), &value)?;
                    let object = self.stack.pop().ok_or(VmError::ValidationException)?;
                    if let Object(object_ref) = object {
                        object_ref.set_field(index, value);
                    } else {
                        return Err(VmError::ValidationException);
                    }
                }

                OpCode::Getfield => {
                    let field_index = instruction.arguments_u16(0)?;
                    let field_reference = self.get_constant_field_reference(field_index)?;

                    // TODO: validate class? How do super classes work?
                    let (index, field) =
                        Self::get_field(self.class_and_method.class, field_reference.field_name)?;

                    let object = self.stack.pop().ok_or(VmError::ValidationException)?;
                    if let Object(object_ref) = object {
                        let field_value = object_ref.get_field(index);
                        Self::validate_type(vm, field.type_descriptor.clone(), &field_value)?;
                        self.stack.push(field_value);
                    } else {
                        return Err(VmError::ValidationException);
                    }
                }

                OpCode::Iadd => self.execute_int_math(vm, |a, b| Ok(a + b))?,
                OpCode::Isub => self.execute_int_math(vm, |a, b| Ok(a - b))?,
                OpCode::Imul => self.execute_int_math(vm, |a, b| Ok(a * b))?,
                OpCode::Idiv => self.execute_int_math(vm, |a, b| match b {
                    0 => Err(VmError::ArithmeticException),
                    _ => Ok(a / b),
                })?,
                OpCode::Irem => self.execute_int_math(vm, |a, b| match b {
                    0 => Err(VmError::ArithmeticException),
                    _ => Ok(a % b),
                })?,
                OpCode::Iand => self.execute_int_math(vm, |a, b| Ok(a & b))?,
                OpCode::Ior => self.execute_int_math(vm, |a, b| Ok(a | b))?,
                OpCode::Ixor => self.execute_int_math(vm, |a, b| Ok(a ^ b))?,

                OpCode::Iinc => {
                    let index = instruction.argument_u8(0)?.into_usize_safe();
                    let local = self.get_local_int_as_int(vm, index)?;
                    let constant = instruction.argument_i8(1)?;
                    self.locals[index] = Value::Int(local + constant as i32);
                }

                OpCode::Goto => self.goto(instruction)?,

                OpCode::Ifeq => self.execute_if(vm, instruction, |v| v == 0)?,
                OpCode::Ifne => self.execute_if(vm, instruction, |v| v != 0)?,
                OpCode::Iflt => self.execute_if(vm, instruction, |v| v < 0)?,
                OpCode::Ifle => self.execute_if(vm, instruction, |v| v <= 0)?,
                OpCode::Ifgt => self.execute_if(vm, instruction, |v| v > 0)?,
                OpCode::Ifge => self.execute_if(vm, instruction, |v| v >= 0)?,

                OpCode::If_icmpeq => self.execute_if_icmp(vm, instruction, |a, b| a == b)?,
                OpCode::If_icmpne => self.execute_if_icmp(vm, instruction, |a, b| a != b)?,
                OpCode::If_icmplt => self.execute_if_icmp(vm, instruction, |a, b| a < b)?,
                OpCode::If_icmple => self.execute_if_icmp(vm, instruction, |a, b| a <= b)?,
                OpCode::If_icmpgt => self.execute_if_icmp(vm, instruction, |a, b| a > b)?,
                OpCode::If_icmpge => self.execute_if_icmp(vm, instruction, |a, b| a >= b)?,

                _ => {
                    warn!("Unsupported op code: {}", instruction.op_code);
                    return Err(VmError::NotImplemented);
                }
            }
        }
    }

    fn invoke_method(
        &mut self,
        vm: &mut Vm<'a>,
        stack: &mut Stack<'a>,
        instruction: Instruction,
        kind: InvokeKind,
    ) -> Result<(), VmError> {
        let class_and_method = self.get_method_to_invoke(vm, instruction, kind)?;
        let (receiver, params, new_stack_len) =
            self.get_method_receiver_and_params(&class_and_method)?;
        self.stack.truncate(new_stack_len);

        let method_return_type = class_and_method.return_type();
        let result = vm.invoke(stack, class_and_method, receiver, params)?;

        Self::validate_type_opt(vm, method_return_type, &result)?;
        if let Some(value) = result {
            self.stack.push(value);
        }
        Ok(())
    }

    fn get_field(
        class: &'a Class,
        field_name: &str,
    ) -> Result<(usize, &'a ClassFileField), VmError> {
        class
            .find_field(field_name)
            .ok_or(VmError::FieldNotFoundException(
                class.name.to_string(),
                field_name.to_string(),
            ))
    }

    fn pop_int(stack: &mut Vec<Value<'a>>, vm: &Vm) -> Result<i32, VmError> {
        let value = stack.pop().ok_or(VmError::ValidationException)?;
        Self::validate_type(vm, Base(BaseType::Int), &value)?;
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
        vm: &Vm<'a>,
        instruction: Instruction,
        kind: InvokeKind,
    ) -> Result<ClassAndMethod<'a>, VmError> {
        let constant_index = instruction.arguments_u16(0)?;
        let method_reference = self.get_constant_method_reference(constant_index)?;

        let class = vm.get_class(method_reference.class_name)?;
        let method = match kind {
            InvokeKind::Special | InvokeKind::Static => Self::get_method(class, method_reference)?,
            InvokeKind::Virtual => Self::get_virtual_method(class, method_reference)?,
        };
        Ok(ClassAndMethod { class, method })
    }

    fn get_method<'b>(
        class: &'b Class<'a>,
        method_reference: MethodReference,
    ) -> Result<&'b ClassFileMethod, VmError> {
        class
            .find_method(
                method_reference.method_name,
                method_reference.type_descriptor,
            )
            .ok_or(VmError::MethodNotFoundException(
                class.name.to_string(),
                method_reference.method_name.to_string(),
                method_reference.type_descriptor.to_string(),
            ))
    }

    fn get_virtual_method<'b>(
        class: &'b Class<'a>,
        method_reference: MethodReference,
    ) -> Result<&'b ClassFileMethod, VmError> {
        let mut curr_class = class;
        loop {
            if let Some(method) = curr_class.find_method(
                method_reference.method_name,
                method_reference.type_descriptor,
            ) {
                return Ok(method);
            }

            if let Some(superclass) = class.superclass {
                curr_class = superclass;
            } else {
                return Err(VmError::MethodNotFoundException(
                    class.name.to_string(),
                    method_reference.method_name.to_string(),
                    method_reference.type_descriptor.to_string(),
                ));
            }
        }
    }

    fn get_method_receiver_and_params(
        &self,
        class_and_method: &ClassAndMethod<'a>,
    ) -> Result<(Option<ObjectRef<'a>>, Vec<Value<'a>>, usize), VmError> {
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
                class_and_method.class,
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
        _expected_class: &Class,
    ) -> Result<ObjectRef<'a>, VmError> {
        let receiver = self.stack.get(index).ok_or(VmError::ValidationException)?;
        match receiver {
            Object(object) => {
                // TODO: here we should check "instanceof" the expected class of a subclass
                Ok(object)
            }
            _ => Err(VmError::ValidationException),
        }
    }

    fn validate_type_opt(
        vm: &Vm,
        expected_type: Option<FieldType>,
        value: &Option<Value<'a>>,
    ) -> Result<(), VmError> {
        match expected_type {
            None => match value {
                None => Ok(()),
                Some(_) => Err(VmError::ValidationException),
            },
            Some(expected_type) => match value {
                None => Err(VmError::ValidationException),
                Some(value) => Self::validate_type(vm, expected_type, value),
            },
        }
    }

    fn validate_type(vm: &Vm, expected_type: FieldType, value: &Value) -> Result<(), VmError> {
        if value.matches_type(expected_type, &vm.class_loader) {
            Ok(())
        } else {
            Err(VmError::ValidationException)
        }
    }

    fn get_local_int(&self, vm: &Vm, index: usize) -> Result<Value<'a>, VmError> {
        // TODO: short, char, byte should (probably?) to be modelled as int
        let variable = self.locals.get(index).ok_or(VmError::ValidationException)?;
        Self::validate_type(vm, Base(BaseType::Int), variable)?;
        Ok(variable.clone())
    }

    fn get_local_int_as_int(&self, vm: &Vm, index: usize) -> Result<i32, VmError> {
        let value = self.get_local_int(vm, index)?;
        match value {
            Value::Int(the_int) => Ok(the_int),
            _ => Err(VmError::ValidationException),
        }
    }

    fn execute_int_math<T>(&mut self, vm: &mut Vm, evaluator: T) -> Result<(), VmError>
    where
        T: FnOnce(i32, i32) -> Result<i32, VmError>,
    {
        let val2 = Self::pop_int(&mut self.stack, vm)?;
        let val1 = Self::pop_int(&mut self.stack, vm)?;
        let result = evaluator(val1, val2)?;
        self.stack.push(Value::Int(result));
        Ok(())
    }

    fn goto(&mut self, instruction: Instruction) -> Result<(), VmError> {
        let offset = instruction.arguments_i16(0)?;
        let new_pc = (self.pc - instruction.length()) as i32 + offset as i32;
        self.pc = usize::try_from(new_pc).map_err(|_| VmError::ValidationException)?;
        Ok(())
    }

    fn execute_if<T>(
        &mut self,
        vm: &mut Vm,
        instruction: Instruction,
        comparator: T,
    ) -> Result<(), VmError>
    where
        T: FnOnce(i32) -> bool,
    {
        let value = Self::pop_int(&mut self.stack, vm)?;
        if comparator(value) {
            self.goto(instruction)
        } else {
            Ok(())
        }
    }

    fn execute_if_icmp<T>(
        &mut self,
        vm: &mut Vm,
        instruction: Instruction,
        comparator: T,
    ) -> Result<(), VmError>
    where
        T: FnOnce(i32, i32) -> bool,
    {
        let val2 = Self::pop_int(&mut self.stack, vm)?;
        let val1 = Self::pop_int(&mut self.stack, vm)?;
        if comparator(val1, val2) {
            self.goto(instruction)
        } else {
            Ok(())
        }
    }

    fn execute_aload(&mut self, index: usize) -> Result<(), VmError> {
        let local = self.locals.get(index).ok_or(VmError::ValidationException)?;
        match local {
            Object(_) => {
                self.stack.push(local.clone());
                Ok(())
            }
            _ => Err(VmError::ValidationException),
        }
    }

    fn execute_astore(&mut self, index: usize) -> Result<(), VmError> {
        let value = self.stack.pop().ok_or(VmError::ValidationException)?;
        match value {
            Object(_) => {
                self.locals[index] = value;
                Ok(())
            }
            _ => Err(VmError::ValidationException),
        }
    }

    fn debug_start_execution(&self) {
        debug!(
            "starting execution of method {}::{} - locals are {:?}",
            self.class_and_method.class.name, self.class_and_method.method.name, self.locals
        )
    }

    fn debug_print_status(&self, instruction: &Instruction) {
        debug!("FRAME STATUS: pc: {}", self.pc);
        debug!("  stack:");
        for stack_entry in self.stack.iter() {
            debug!("  - {:?}", stack_entry);
        }
        debug!("  locals:");
        for local_variable in self.locals.iter() {
            debug!("  - {:?}", local_variable);
        }
        debug!("  next instruction: {}", instruction)
    }

    fn debug_done_execution(&self, result: Option<&Value>) {
        debug!(
            "completed execution of method {}::{} - result is {:?}",
            self.class_and_method.class.name, self.class_and_method.method.name, result
        )
    }
}

#[derive(Debug, Default)]
pub struct Vm<'a> {
    class_allocator: ClassAllocator<'a>,
    class_loader: ClassLoader<'a>,
    object_allocator: ObjectAllocator<'a>,
    pub printed: Vec<Value<'a>>, // Temporary, used for testing purposes
}

impl<'a> Vm<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn load_class(&mut self, class_file: ClassFile) -> Result<(), VmError> {
        let class = self
            .class_allocator
            .allocate(class_file, &self.class_loader)?;
        self.class_loader.register_class(class);
        Ok(())
    }

    pub fn find_class<'b>(&'b self, class_name: &str) -> Option<ClassRef<'a>> {
        self.class_loader.find_class_by_name(class_name)
    }

    pub fn get_class(&self, class_name: &str) -> Result<ClassRef<'a>, VmError> {
        self.find_class(class_name)
            .ok_or(VmError::ClassNotFoundException(class_name.to_string()))
    }

    pub fn find_class_method(
        &self,
        class_name: &str,
        method_name: &str,
        method_type_descriptor: &str,
    ) -> Option<ClassAndMethod<'a>> {
        self.find_class(class_name).and_then(|class| {
            class
                .find_method(method_name, method_type_descriptor)
                .map(|method| ClassAndMethod { class, method })
        })
    }

    // TODO: do we need it?
    pub fn allocate_stack(&self) -> Stack<'a> {
        Stack::new()
    }

    pub fn invoke(
        &mut self,
        stack: &mut Stack<'a>,
        class_and_method: ClassAndMethod<'a>,
        object: Option<ObjectRef<'a>>,
        args: Vec<Value<'a>>,
    ) -> Result<Option<Value<'a>>, VmError> {
        if class_and_method.method.is_native() {
            return if class_and_method.class.name.starts_with("rjvm/")
                && class_and_method.method.name == "tempPrint"
            {
                let arg = args.get(0).ok_or(VmError::ValidationException)?;
                info!("TEMP implementation of native method: printing value {arg:?}");
                self.printed.push(arg.clone());
                Ok(None)
            } else {
                Err(VmError::NotImplemented)
            };
        }

        let frame = stack.add_frame(class_and_method, object, args)?;
        let result = frame.borrow_mut().execute(self, stack);
        stack.pop_frame()?;
        result
    }

    pub fn new_object(&mut self, class_name: &str) -> Result<ObjectRef<'a>, VmError> {
        debug!("allocating new instance of {}", class_name);

        let class = self.get_class(class_name)?;
        let instance = self.object_allocator.allocate(class);
        Ok(instance)
    }

    pub fn debug_stats(&self) {
        debug!(
            "VM classes={:?}, objects = {:?}",
            self.class_allocator, self.object_allocator
        )
    }
}
