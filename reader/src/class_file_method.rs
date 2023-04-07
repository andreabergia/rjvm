use std::{fmt, fmt::Formatter};

use crate::field_type::BaseType;
use crate::{
    attribute::Attribute, field_type::FieldType, instruction::Instruction,
    method_descriptor::MethodDescriptor, method_flags::MethodFlags,
};

#[derive(Debug, PartialEq)]
pub struct ClassFileMethod {
    pub flags: MethodFlags,
    pub name: String,
    pub type_descriptor: String,
    pub parsed_type_descriptor: MethodDescriptor,
    pub attributes: Vec<Attribute>,
    pub code: Option<ClassFileMethodCode>,
    pub deprecated: bool,
}

impl fmt::Display for ClassFileMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{:?} {}: {}{}",
            self.flags,
            self.name,
            self.parsed_type_descriptor,
            if self.deprecated { " (deprecated)" } else { "" }
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

#[derive(Debug, Default, PartialEq)]
pub struct ClassFileMethodCode {
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: Vec<u8>,
    pub exception_table: Vec<u8>, // TODO: replace with some proper struct
    pub attributes: Vec<Attribute>, // TODO: replace with some proper struct
    pub line_number_table: Option<LineNumberTable>,
}

impl fmt::Display for ClassFileMethodCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "max_stack = {}, max_locals = {}, exception_table = {:?}, attributes = {:?}, instructions:",
            self.max_stack, self.max_locals, self.exception_table, self.attributes,
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

#[derive(Debug, PartialEq)]
pub struct LineNumberTable {
    entries: Vec<LineNumberTableEntry>,
}

impl LineNumberTable {
    pub fn new(entries: Vec<LineNumberTableEntry>) -> Self {
        Self { entries }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ProgramCounter(pub u16);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LineNumber(pub u16);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LineNumberTableEntry {
    pub program_counter: ProgramCounter,
    pub line_number: LineNumber,
}

impl LineNumberTableEntry {
    pub fn new(program_counter: ProgramCounter, line_number: LineNumber) -> Self {
        Self {
            program_counter,
            line_number,
        }
    }
}
