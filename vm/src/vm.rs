use std::cell::RefCell;
use std::rc::Rc;

use log::{debug, info, warn};

use rjvm_reader::{
    class_file::ClassFile, class_file_field::ClassFileField, class_file_method::ClassFileMethod,
    constant_pool::ConstantPoolEntry, field_type::BaseType, field_type::FieldType,
    field_type::FieldType::Base, instruction::Instruction, method_flags::MethodFlags,
};
use rjvm_utils::type_conversion::ToUsizeSafe;

use crate::class::{ClassId, ClassRef};
use crate::class_allocator::{ClassAllocator, ClassResolver};
use crate::gc::ObjectAllocator;
use crate::value::ObjectRef;
use crate::value::Value::{Double, Float, Int, Long};
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
    #[allow(dead_code)]
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

#[derive(Clone, Copy)]
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
            let (instruction, new_address) =
                Instruction::parse(self.code, self.pc).map_err(|_| VmError::ValidationException)?;
            self.debug_print_status(&instruction);
            self.pc = new_address;

            match instruction {
                Instruction::Aconst_null => self.stack.push(Value::Null),
                Instruction::Aload(index) => self.execute_aload(index.into_usize_safe())?,
                Instruction::Aload_0 => self.execute_aload(0)?,
                Instruction::Aload_1 => self.execute_aload(1)?,
                Instruction::Aload_2 => self.execute_aload(2)?,
                Instruction::Aload_3 => self.execute_aload(3)?,

                Instruction::Astore(index) => self.execute_astore(index.into_usize_safe())?,
                Instruction::Astore_0 => self.execute_astore(0)?,
                Instruction::Astore_1 => self.execute_astore(1)?,
                Instruction::Astore_2 => self.execute_astore(2)?,
                Instruction::Astore_3 => self.execute_astore(3)?,

                Instruction::Iload(index) => self.execute_iload(index.into_usize_safe())?,
                Instruction::Iload_0 => self.execute_iload(0)?,
                Instruction::Iload_1 => self.execute_iload(1)?,
                Instruction::Iload_2 => self.execute_iload(2)?,
                Instruction::Iload_3 => self.execute_iload(3)?,

                Instruction::Istore(index) => self.execute_istore(index.into_usize_safe())?,
                Instruction::Istore_0 => self.execute_istore(0)?,
                Instruction::Istore_1 => self.execute_istore(1)?,
                Instruction::Istore_2 => self.execute_istore(2)?,
                Instruction::Istore_3 => self.execute_istore(3)?,

                Instruction::Iconst_m1 => self.stack.push(Int(-1)),
                Instruction::Iconst_0 => self.stack.push(Int(0)),
                Instruction::Iconst_1 => self.stack.push(Int(1)),
                Instruction::Iconst_2 => self.stack.push(Int(2)),
                Instruction::Iconst_3 => self.stack.push(Int(3)),
                Instruction::Iconst_4 => self.stack.push(Int(4)),
                Instruction::Iconst_5 => self.stack.push(Int(5)),

                Instruction::Lload(index) => self.execute_lload(index.into_usize_safe())?,
                Instruction::Lload_0 => self.execute_lload(0)?,
                Instruction::Lload_1 => self.execute_lload(1)?,
                Instruction::Lload_2 => self.execute_lload(2)?,
                Instruction::Lload_3 => self.execute_lload(3)?,

                Instruction::Lstore(index) => self.execute_lstore(index.into_usize_safe())?,
                Instruction::Lstore_0 => self.execute_lstore(0)?,
                Instruction::Lstore_1 => self.execute_lstore(1)?,
                Instruction::Lstore_2 => self.execute_lstore(2)?,
                Instruction::Lstore_3 => self.execute_lstore(3)?,

                Instruction::Ldc(index) => self.execute_ldc(index as u16)?,
                Instruction::Ldc_w(index) => self.execute_ldc(index)?,
                Instruction::Ldc2_w(index) => self.execute_ldc_long_double(index)?,

                Instruction::Fload(index) => self.execute_fload(index.into_usize_safe())?,
                Instruction::Fload_0 => self.execute_fload(0)?,
                Instruction::Fload_1 => self.execute_fload(1)?,
                Instruction::Fload_2 => self.execute_fload(2)?,
                Instruction::Fload_3 => self.execute_fload(3)?,

                Instruction::Fstore(index) => self.execute_fstore(index.into_usize_safe())?,
                Instruction::Fstore_0 => self.execute_fstore(0)?,
                Instruction::Fstore_1 => self.execute_fstore(1)?,
                Instruction::Fstore_2 => self.execute_fstore(2)?,
                Instruction::Fstore_3 => self.execute_fstore(3)?,

                Instruction::Dload(index) => self.execute_dload(index.into_usize_safe())?,
                Instruction::Dload_0 => self.execute_dload(0)?,
                Instruction::Dload_1 => self.execute_dload(1)?,
                Instruction::Dload_2 => self.execute_dload(2)?,
                Instruction::Dload_3 => self.execute_dload(3)?,

                Instruction::Dstore(index) => self.execute_dstore(index.into_usize_safe())?,
                Instruction::Dstore_0 => self.execute_dstore(0)?,
                Instruction::Dstore_1 => self.execute_dstore(1)?,
                Instruction::Dstore_2 => self.execute_dstore(2)?,
                Instruction::Dstore_3 => self.execute_dstore(3)?,

                Instruction::I2b => self.coerce_int(vm, |value| Int((value as i8) as i32))?,
                Instruction::I2c => self.coerce_int(vm, |value| Int((value as u16) as i32))?,
                Instruction::I2s => self.coerce_int(vm, |value| Int((value as i16) as i32))?,
                Instruction::I2f => self.coerce_int(vm, |value| Float(value as f32))?,
                Instruction::I2l => self.coerce_int(vm, |value| Long(value as i64))?,
                Instruction::I2d => self.coerce_int(vm, |value| Double(value as f64))?,

                Instruction::New(constant_index) => {
                    let new_object_class_name =
                        self.get_constant_class_reference(constant_index)?;
                    let new_object = vm.new_object(new_object_class_name)?;
                    self.stack.push(Object(new_object));
                }

                Instruction::Dup => {
                    let stack_head = self.stack.last().ok_or(VmError::ValidationException)?;
                    self.stack.push(stack_head.clone());
                }
                Instruction::Pop => {
                    self.stack.pop().ok_or(VmError::ValidationException)?;
                }
                Instruction::Bipush(byte_value) => self.stack.push(Int(byte_value as i32)),

                Instruction::Invokespecial(constant_index) => {
                    self.invoke_method(vm, stack, constant_index, InvokeKind::Special)?
                }
                Instruction::Invokestatic(constant_index) => {
                    self.invoke_method(vm, stack, constant_index, InvokeKind::Static)?
                }
                Instruction::Invokevirtual(constant_index) => {
                    self.invoke_method(vm, stack, constant_index, InvokeKind::Virtual)?
                }

                Instruction::Return => {
                    if !self.class_and_method.is_void() {
                        return Err(VmError::ValidationException);
                    }
                    self.debug_done_execution(None);
                    return Ok(None);
                }
                Instruction::Ireturn => {
                    if !self.class_and_method.returns(Base(BaseType::Int)) {
                        return Err(VmError::ValidationException);
                    }
                    let result = self.stack.pop().ok_or(VmError::ValidationException)?;
                    self.debug_done_execution(Some(&result));
                    return Ok(Some(result));
                }

                Instruction::Putfield(field_index) => {
                    let field_reference = self.get_constant_field_reference(field_index)?;

                    let (index, field) =
                        Self::get_field(self.class_and_method.class, field_reference)?;

                    let value = self.stack.pop().ok_or(VmError::ValidationException)?;
                    Self::validate_type(vm, field.type_descriptor.clone(), &value)?;
                    let object = self.stack.pop().ok_or(VmError::ValidationException)?;
                    if let Object(object_ref) = object {
                        object_ref.set_field(index, value);
                    } else {
                        return Err(VmError::ValidationException);
                    }
                }

                Instruction::Getfield(field_index) => {
                    let field_reference = self.get_constant_field_reference(field_index)?;

                    let (index, field) =
                        Self::get_field(self.class_and_method.class, field_reference)?;

                    let object = self.stack.pop().ok_or(VmError::ValidationException)?;
                    if let Object(object_ref) = object {
                        let field_value = object_ref.get_field(index);
                        Self::validate_type(vm, field.type_descriptor.clone(), &field_value)?;
                        self.stack.push(field_value);
                    } else {
                        return Err(VmError::ValidationException);
                    }
                }

                Instruction::Iadd => self.execute_int_math(vm, |a, b| Ok(a + b))?,
                Instruction::Isub => self.execute_int_math(vm, |a, b| Ok(a - b))?,
                Instruction::Imul => self.execute_int_math(vm, |a, b| Ok(a * b))?,
                Instruction::Idiv => self.execute_int_math(vm, |a, b| match b {
                    0 => Err(VmError::ArithmeticException),
                    _ => Ok(a / b),
                })?,
                Instruction::Irem => self.execute_int_math(vm, |a, b| match b {
                    0 => Err(VmError::ArithmeticException),
                    _ => Ok(a % b),
                })?,
                Instruction::Iand => self.execute_int_math(vm, |a, b| Ok(a & b))?,
                Instruction::Ior => self.execute_int_math(vm, |a, b| Ok(a | b))?,
                Instruction::Ixor => self.execute_int_math(vm, |a, b| Ok(a ^ b))?,

                Instruction::Iinc(index, constant) => {
                    let index = index.into_usize_safe();
                    let local = self.get_local_int_as_int(vm, index)?;
                    self.locals[index] = Int(local + constant as i32);
                }

                Instruction::Ladd => self.execute_long_math(vm, |a, b| Ok(a + b))?,
                Instruction::Lsub => self.execute_long_math(vm, |a, b| Ok(a - b))?,
                Instruction::Lmul => self.execute_long_math(vm, |a, b| Ok(a * b))?,
                Instruction::Ldiv => self.execute_long_math(vm, |a, b| match b {
                    0 => Err(VmError::ArithmeticException),
                    _ => Ok(a / b),
                })?,
                Instruction::Lrem => self.execute_long_math(vm, |a, b| match b {
                    0 => Err(VmError::ArithmeticException),
                    _ => Ok(a % b),
                })?,
                Instruction::Land => self.execute_long_math(vm, |a, b| Ok(a & b))?,
                Instruction::Lor => self.execute_long_math(vm, |a, b| Ok(a | b))?,
                Instruction::Lxor => self.execute_long_math(vm, |a, b| Ok(a ^ b))?,

                Instruction::Fadd => self.execute_float_math(vm, |a, b| Ok(a + b))?,
                Instruction::Fsub => self.execute_float_math(vm, |a, b| Ok(a - b))?,
                Instruction::Fmul => self.execute_float_math(vm, |a, b| Ok(a * b))?,
                Instruction::Fdiv => self.execute_float_math(vm, |a, b| {
                    Ok(
                        if Self::is_double_division_returning_nan(a as f64, b as f64) {
                            f32::NAN
                        } else {
                            a / b
                        },
                    )
                })?,
                Instruction::Frem => self.execute_float_math(vm, |a, b| {
                    Ok(
                        if Self::is_double_division_returning_nan(a as f64, b as f64) {
                            f32::NAN
                        } else {
                            a % b
                        },
                    )
                })?,

                Instruction::Dadd => self.execute_double_math(vm, |a, b| Ok(a + b))?,
                Instruction::Dsub => self.execute_double_math(vm, |a, b| Ok(a - b))?,
                Instruction::Dmul => self.execute_double_math(vm, |a, b| Ok(a * b))?,
                Instruction::Ddiv => self.execute_double_math(vm, |a, b| {
                    Ok(if Self::is_double_division_returning_nan(a, b) {
                        f64::NAN
                    } else {
                        a / b
                    })
                })?,
                Instruction::Drem => self.execute_double_math(vm, |a, b| {
                    Ok(if Self::is_double_division_returning_nan(a, b) {
                        f64::NAN
                    } else {
                        a % b
                    })
                })?,

                Instruction::Goto(jump_address) => self.goto(jump_address),

                Instruction::Ifeq(jump_address) => self.execute_if(vm, jump_address, |v| v == 0)?,
                Instruction::Ifne(jump_address) => self.execute_if(vm, jump_address, |v| v != 0)?,
                Instruction::Iflt(jump_address) => self.execute_if(vm, jump_address, |v| v < 0)?,
                Instruction::Ifle(jump_address) => self.execute_if(vm, jump_address, |v| v <= 0)?,
                Instruction::Ifgt(jump_address) => self.execute_if(vm, jump_address, |v| v > 0)?,
                Instruction::Ifge(jump_address) => self.execute_if(vm, jump_address, |v| v >= 0)?,

                Instruction::If_icmpeq(jump_address) => {
                    self.execute_if_icmp(vm, jump_address, |a, b| a == b)?
                }
                Instruction::If_icmpne(jump_address) => {
                    self.execute_if_icmp(vm, jump_address, |a, b| a != b)?
                }
                Instruction::If_icmplt(jump_address) => {
                    self.execute_if_icmp(vm, jump_address, |a, b| a < b)?
                }
                Instruction::If_icmple(jump_address) => {
                    self.execute_if_icmp(vm, jump_address, |a, b| a <= b)?
                }
                Instruction::If_icmpgt(jump_address) => {
                    self.execute_if_icmp(vm, jump_address, |a, b| a > b)?
                }
                Instruction::If_icmpge(jump_address) => {
                    self.execute_if_icmp(vm, jump_address, |a, b| a >= b)?
                }

                _ => {
                    warn!("Unsupported instruction: {:?}", instruction);
                    return Err(VmError::NotImplemented);
                }
            }
        }
    }

    fn invoke_method(
        &mut self,
        vm: &mut Vm<'a>,
        stack: &mut Stack<'a>,
        constant_index: u16,
        kind: InvokeKind,
    ) -> Result<(), VmError> {
        let static_method_reference =
            self.get_method_to_invoke_statically(vm, constant_index, kind)?;
        let (receiver, params, new_stack_len) =
            self.get_method_receiver_and_params(&static_method_reference)?;
        let class_and_method = match kind {
            InvokeKind::Virtual => {
                Self::resolve_virtual_method(vm, receiver, static_method_reference)?
            }
            _ => static_method_reference,
        };
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
        field_reference: FieldReference,
    ) -> Result<(usize, &'a ClassFileField), VmError> {
        class
            .find_field(field_reference.class_name, field_reference.field_name)
            .ok_or(VmError::FieldNotFoundException(
                field_reference.class_name.to_string(),
                field_reference.field_name.to_string(),
            ))
    }

    fn pop_int(&mut self, _vm: &Vm) -> Result<i32, VmError> {
        let value = self.stack.pop().ok_or(VmError::ValidationException)?;
        match value {
            Int(int) => Ok(int),
            _ => Err(VmError::ValidationException),
        }
    }

    fn pop_long(&mut self, _vm: &Vm) -> Result<i64, VmError> {
        let value = self.stack.pop().ok_or(VmError::ValidationException)?;
        match value {
            Long(long) => Ok(long),
            _ => Err(VmError::ValidationException),
        }
    }

    fn pop_float(&mut self, _vm: &Vm) -> Result<f32, VmError> {
        let value = self.stack.pop().ok_or(VmError::ValidationException)?;
        match value {
            Float(float) => Ok(float),
            _ => Err(VmError::ValidationException),
        }
    }

    fn pop_double(&mut self, _vm: &Vm) -> Result<f64, VmError> {
        let value = self.stack.pop().ok_or(VmError::ValidationException)?;
        match value {
            Double(double) => Ok(double),
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

    fn get_method_to_invoke_statically(
        &self,
        vm: &Vm<'a>,
        constant_index: u16,
        kind: InvokeKind,
    ) -> Result<ClassAndMethod<'a>, VmError> {
        let method_reference = self.get_constant_method_reference(constant_index)?;

        let class = vm.get_class(method_reference.class_name)?;
        match kind {
            InvokeKind::Special | InvokeKind::Static => {
                Self::get_method_of_class(class, method_reference)
                    .map(|method| ClassAndMethod { class, method })
            }
            InvokeKind::Virtual => Self::get_method_checking_superclasses(class, method_reference),
        }
    }

    fn get_method_of_class<'b>(
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

    fn get_method_checking_superclasses<'b>(
        class: &'b Class<'a>,
        method_reference: MethodReference,
    ) -> Result<ClassAndMethod<'b>, VmError> {
        let mut curr_class = class;
        loop {
            if let Some(method) = curr_class.find_method(
                method_reference.method_name,
                method_reference.type_descriptor,
            ) {
                return Ok(ClassAndMethod {
                    class: curr_class,
                    method,
                });
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

    fn resolve_virtual_method(
        vm: &Vm<'a>,
        receiver: Option<ObjectRef>,
        class_and_method: ClassAndMethod,
    ) -> Result<ClassAndMethod<'a>, VmError> {
        match receiver {
            None => Err(VmError::ValidationException),
            Some(receiver) => {
                let receiver_class = vm.get_class_by_id(receiver.class_id)?;
                let resolved_method = Self::get_method_checking_superclasses(
                    receiver_class,
                    MethodReference {
                        class_name: &class_and_method.class.name,
                        method_name: &class_and_method.method.name,
                        type_descriptor: &class_and_method.method.type_descriptor,
                    },
                )?;
                debug!(
                    "resolved virtual method {}.{}:{} on object of class {}: using version of class {}",
                    class_and_method.class.name,
                    class_and_method.method.name,
                    class_and_method.method.type_descriptor,
                    receiver_class.name,
                    resolved_method.class.name,
                );
                Ok(resolved_method)
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
        let variable = self.locals.get(index).ok_or(VmError::ValidationException)?;
        Self::validate_type(vm, Base(BaseType::Int), variable)?;
        Ok(variable.clone())
    }

    fn get_local_int_as_int(&self, vm: &Vm, index: usize) -> Result<i32, VmError> {
        let value = self.get_local_int(vm, index)?;
        match value {
            Int(the_int) => Ok(the_int),
            _ => Err(VmError::ValidationException),
        }
    }

    fn execute_int_math<T>(&mut self, vm: &mut Vm, evaluator: T) -> Result<(), VmError>
    where
        T: FnOnce(i32, i32) -> Result<i32, VmError>,
    {
        let val2 = self.pop_int(vm)?;
        let val1 = self.pop_int(vm)?;
        let result = evaluator(val1, val2)?;
        self.stack.push(Int(result));
        Ok(())
    }

    fn execute_long_math<T>(&mut self, vm: &mut Vm, evaluator: T) -> Result<(), VmError>
    where
        T: FnOnce(i64, i64) -> Result<i64, VmError>,
    {
        let val2 = self.pop_long(vm)?;
        let val1 = self.pop_long(vm)?;
        let result = evaluator(val1, val2)?;
        self.stack.push(Long(result));
        Ok(())
    }

    fn is_double_division_returning_nan(a: f64, b: f64) -> bool {
        a.is_nan()
            || b.is_nan()
            || (a.is_infinite() && b.is_infinite())
            || ((a == 0f64 || a == -0f64) && (b == 0f64 || b == -0f64))
    }

    fn execute_float_math<T>(&mut self, vm: &mut Vm, evaluator: T) -> Result<(), VmError>
    where
        T: FnOnce(f32, f32) -> Result<f32, VmError>,
    {
        let val2 = self.pop_float(vm)?;
        let val1 = self.pop_float(vm)?;
        let result = evaluator(val1, val2)?;
        self.stack.push(Float(result));
        Ok(())
    }

    fn execute_double_math<T>(&mut self, vm: &mut Vm, evaluator: T) -> Result<(), VmError>
    where
        T: FnOnce(f64, f64) -> Result<f64, VmError>,
    {
        let val2 = self.pop_double(vm)?;
        let val1 = self.pop_double(vm)?;
        let result = evaluator(val1, val2)?;
        self.stack.push(Double(result));
        Ok(())
    }

    fn coerce_int<T>(&mut self, vm: &mut Vm, evaluator: T) -> Result<(), VmError>
    where
        T: FnOnce(i32) -> Value<'a>,
    {
        let value = self.pop_int(vm)?;
        let coerced = evaluator(value);
        self.stack.push(coerced);
        Ok(())
    }

    fn goto(&mut self, jump_address: u16) {
        self.pc = jump_address.into_usize_safe();
    }

    fn execute_if<T>(
        &mut self,
        vm: &mut Vm,
        jump_address: u16,
        comparator: T,
    ) -> Result<(), VmError>
    where
        T: FnOnce(i32) -> bool,
    {
        let value = self.pop_int(vm)?;
        if comparator(value) {
            self.goto(jump_address);
        }
        Ok(())
    }

    fn execute_if_icmp<T>(
        &mut self,
        vm: &mut Vm,
        jump_address: u16,
        comparator: T,
    ) -> Result<(), VmError>
    where
        T: FnOnce(i32, i32) -> bool,
    {
        let val2 = self.pop_int(vm)?;
        let val1 = self.pop_int(vm)?;
        if comparator(val1, val2) {
            self.goto(jump_address);
        }
        Ok(())
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

    fn execute_iload(&mut self, index: usize) -> Result<(), VmError> {
        let local = self.locals.get(index).ok_or(VmError::ValidationException)?;
        match local {
            Int(_) => {
                self.stack.push(local.clone());
                Ok(())
            }
            _ => Err(VmError::ValidationException),
        }
    }

    fn execute_istore(&mut self, index: usize) -> Result<(), VmError> {
        let value = self.stack.pop().ok_or(VmError::ValidationException)?;
        match value {
            Int(_) => {
                self.locals[index] = value;
                Ok(())
            }
            _ => Err(VmError::ValidationException),
        }
    }

    fn execute_lload(&mut self, index: usize) -> Result<(), VmError> {
        let local = self.locals.get(index).ok_or(VmError::ValidationException)?;
        match local {
            Long(_) => {
                self.stack.push(local.clone());
                Ok(())
            }
            _ => Err(VmError::ValidationException),
        }
    }

    fn execute_lstore(&mut self, index: usize) -> Result<(), VmError> {
        let value = self.stack.pop().ok_or(VmError::ValidationException)?;
        match value {
            Long(_) => {
                self.locals[index] = value;
                Ok(())
            }
            _ => Err(VmError::ValidationException),
        }
    }

    fn execute_fload(&mut self, index: usize) -> Result<(), VmError> {
        let local = self.locals.get(index).ok_or(VmError::ValidationException)?;
        match local {
            Float(_) => {
                self.stack.push(local.clone());
                Ok(())
            }
            _ => Err(VmError::ValidationException),
        }
    }

    fn execute_fstore(&mut self, index: usize) -> Result<(), VmError> {
        let value = self.stack.pop().ok_or(VmError::ValidationException)?;
        match value {
            Float(_) => {
                self.locals[index] = value;
                Ok(())
            }
            _ => Err(VmError::ValidationException),
        }
    }

    fn execute_dload(&mut self, index: usize) -> Result<(), VmError> {
        let local = self.locals.get(index).ok_or(VmError::ValidationException)?;
        match local {
            Double(_) => {
                self.stack.push(local.clone());
                Ok(())
            }
            _ => Err(VmError::ValidationException),
        }
    }

    fn execute_dstore(&mut self, index: usize) -> Result<(), VmError> {
        let value = self.stack.pop().ok_or(VmError::ValidationException)?;
        match value {
            Double(_) => {
                self.locals[index] = value;
                Ok(())
            }
            _ => Err(VmError::ValidationException),
        }
    }

    fn execute_ldc(&mut self, index: u16) -> Result<(), VmError> {
        let constant_value = self.get_constant(index)?;
        match constant_value {
            ConstantPoolEntry::Integer(value) => self.stack.push(Int(*value)),
            ConstantPoolEntry::Float(value) => self.stack.push(Float(*value)),
            // TODO: StringReference
            // TODO: ClassReference
            // TODO: method type or method handle
            _ => return Err(VmError::ValidationException),
        }
        Ok(())
    }

    fn execute_ldc_long_double(&mut self, index: u16) -> Result<(), VmError> {
        let constant_value = self.get_constant(index)?;
        match constant_value {
            ConstantPoolEntry::Long(value) => self.stack.push(Long(*value)),
            ConstantPoolEntry::Double(value) => self.stack.push(Double(*value)),
            _ => return Err(VmError::ValidationException),
        }
        Ok(())
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
        debug!("  next instruction: {:?}", instruction)
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

    pub fn get_class_by_id(&self, class_id: ClassId) -> Result<ClassRef<'a>, VmError> {
        self.class_loader
            .find_class_by_id(class_id)
            .ok_or(VmError::ClassNotFoundException(class_id.to_string()))
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
