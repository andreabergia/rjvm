use std::{fmt, fmt::Formatter};

use typed_arena::Arena;

use rjvm_reader::class_file_method::ClassFileMethodCode;
use rjvm_reader::method_flags::MethodFlags;
use rjvm_utils::type_conversion::ToUsizeSafe;

use crate::{
    call_frame::CallFrame, class_and_method::ClassAndMethod, object::Object,
    stack_trace_element::StackTraceElement, value::Value, vm_error::VmError,
};

// The allocator will allocate and ensure that our call frames are alive while the call stack is.
// Thus, we can do some unsafe magic to avoid Rc<RefCell<>>, which would mess up our code when
// we try to get a stack trace _while_ executing a method.
#[derive(Default)]
pub struct CallStack<'a> {
    frames: Vec<CallFrameReference<'a>>,
    allocator: Arena<CallFrame<'a>>,
}

// SAFETY: The pointer will be valid until the generating call stack is,
// since the pointee it is valid until the arena is.
#[derive(Debug, Clone)]
pub struct CallFrameReference<'a>(*mut CallFrame<'a>);

impl<'a> AsRef<CallFrame<'a>> for CallFrameReference<'a> {
    fn as_ref(&self) -> &CallFrame<'a> {
        unsafe { self.0.as_ref() }.unwrap()
    }
}

impl<'a> AsMut<CallFrame<'a>> for CallFrameReference<'a> {
    fn as_mut(&mut self) -> &mut CallFrame<'a> {
        unsafe { self.0.as_mut() }.unwrap()
    }
}

impl<'a> CallStack<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_frame(
        &mut self,
        class_and_method: ClassAndMethod<'a>,
        receiver: Option<Object<'a>>,
        args: Vec<Value<'a>>,
    ) -> Result<CallFrameReference<'a>, VmError> {
        let code = Self::get_code(&class_and_method, receiver.clone())?;
        let locals = Self::prepare_locals(code, receiver, args);
        let new_frame = self
            .allocator
            .alloc(CallFrame::new(class_and_method, locals));

        let reference = CallFrameReference(new_frame);
        self.frames.push(reference.clone());
        Ok(reference)
    }

    fn get_code<'b>(
        class_and_method: &'b ClassAndMethod,
        receiver: Option<Object>,
    ) -> Result<&'b ClassFileMethodCode, VmError> {
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
        Ok(code)
    }

    fn prepare_locals(
        code: &ClassFileMethodCode,
        receiver: Option<Object<'a>>,
        args: Vec<Value<'a>>,
    ) -> Vec<Value<'a>> {
        let mut locals: Vec<Value<'a>> = receiver
            .map(Value::Object)
            .into_iter()
            .chain(args.into_iter())
            .collect();
        while locals.len() < code.max_locals.into_usize_safe() {
            locals.push(Value::Uninitialized);
        }
        locals
    }

    pub fn pop_frame(&mut self) -> Result<(), VmError> {
        self.frames
            .pop()
            .map(|_| ())
            .ok_or(VmError::ValidationException)
    }

    pub fn get_stack_trace_elements(&self) -> Vec<StackTraceElement<'a>> {
        self.frames
            .iter()
            .rev()
            .map(|frame| frame.as_ref().to_stack_trace_element())
            .collect()
    }

    pub fn gc_roots(&mut self) -> impl Iterator<Item = *mut Object<'a>> {
        let mut roots = vec![];
        roots.extend(
            self.frames
                .iter_mut()
                .map(|frame| frame.as_mut().gc_roots())
                .flatten(),
        );
        roots.into_iter()
    }
}

impl<'a> fmt::Debug for CallStack<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "CallStack{{frames={:?}}}", self.frames)
    }
}
