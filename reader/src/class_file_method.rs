use std::{fmt, fmt::Formatter};

use crate::{
    attribute::Attribute,
    exception_table::ExceptionTable,
    field_type::{BaseType, FieldType},
    instruction::Instruction,
    line_number_table::LineNumberTable,
    method_descriptor::MethodDescriptor,
    method_flags::MethodFlags,
};

/// Models a method in a class
#[derive(Debug, PartialEq)]
pub struct ClassFileMethod {
    pub flags: MethodFlags,
    pub name: String,
    /// The type descriptor in the internal JVM form, i.e. something like (L)I in the unparsed form
    pub type_descriptor: String,
    /// Parsed form of the method descriptor
    pub parsed_type_descriptor: MethodDescriptor,
    /// Generic attributes of the method
    // TODO: replace with some proper struct
    pub attributes: Vec<Attribute>,
    pub code: Option<ClassFileMethodCode>,
    pub deprecated: bool,
    /// List of exceptions in the `throws` clause of the method
    pub thrown_exceptions: Vec<String>,
}

impl fmt::Display for ClassFileMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{:?} {}: {}{} throws {:?}",
            self.flags,
            self.name,
            self.parsed_type_descriptor,
            if self.deprecated { " (deprecated)" } else { "" },
            self.thrown_exceptions,
        )?;
        if let Some(code) = &self.code {
            writeln!(f, "  code: {code}")?;
        }
        write!(f, "  raw_attributes: {:?}", self.attributes)
    }
}

impl ClassFileMethod {
    pub fn is_static(&self) -> bool {
        self.flags.contains(MethodFlags::STATIC)
    }

    pub fn is_native(&self) -> bool {
        self.flags.contains(MethodFlags::NATIVE)
    }

    pub fn is_void(&self) -> bool {
        self.parsed_type_descriptor.return_type.is_none()
    }

    pub fn returns(&self, expected_type: FieldType) -> bool {
        match self.parsed_type_descriptor.return_type {
            Some(FieldType::Base(BaseType::Int))
            | Some(FieldType::Base(BaseType::Short))
            | Some(FieldType::Base(BaseType::Char))
            | Some(FieldType::Base(BaseType::Byte))
            | Some(FieldType::Base(BaseType::Boolean)) => {
                FieldType::Base(BaseType::Int) == expected_type
            }
            _ => self.parsed_type_descriptor.return_type == Some(expected_type),
        }
    }
}

/// Code of a given method
#[derive(Debug, Default, PartialEq)]
pub struct ClassFileMethodCode {
    /// Maximum depth of the stack at any time
    pub max_stack: u16,
    /// Number of local variables used by the method
    pub max_locals: u16,
    /// Raw bytecode
    pub code: Vec<u8>,
    pub exception_table: ExceptionTable,
    pub line_number_table: Option<LineNumberTable>,

    /// Generic unmapped attributes of the code
    // TODO: replace with some proper struct
    pub attributes: Vec<Attribute>,
}

impl fmt::Display for ClassFileMethodCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "max_stack = {}, max_locals = {}, exception_table = {:?}, line_number_table: {:?}, attributes = {:?}, instructions:",
            self.max_stack, self.max_locals, self.exception_table, self.line_number_table, self.attributes,
        )?;

        let instructions = Instruction::parse_instructions(&self.code);
        if let Ok(instructions) = instructions {
            for (address, instruction) in instructions {
                writeln!(f, "    {address:3} {instruction:?}")?;
            }
        } else {
            writeln!(f, "    unparseable code: {:?}", self.code)?;
        }
        Ok(())
    }
}
