use std::fmt;
use std::fmt::Formatter;

use rjvm_utils::buffer::Buffer;

use crate::class_reader_error::ClassReaderError;
use crate::class_reader_error::ClassReaderError::UnsupportedInstruction;
use crate::opcodes::InstructionLength::Fixed;
use crate::opcodes::{InstructionLength, OpCode};

#[derive(Debug, PartialEq)]
pub struct Instruction<'a> {
    pub op_code: OpCode,
    pub arguments: &'a [u8],
}

impl<'a> fmt::Display for Instruction<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.arguments.is_empty() {
            write!(f, "{}", self.op_code)
        } else {
            write!(f, "{} {:?}", self.op_code, self.arguments)
        }
    }
}

impl<'a> Instruction<'a> {
    pub fn parse_instruction(raw_code: &'a [u8], index: usize) -> Result<Self, ClassReaderError> {
        let op_byte = *raw_code
            .get(index)
            .ok_or(ClassReaderError::InvalidClassData(format!(
                "cannot read instruction at offset {}",
                index
            )))?;
        let op_code = OpCode::try_from(op_byte).map_err(|_| {
            ClassReaderError::InvalidClassData(format!("invalid op code: {op_byte:#04x}"))
        })?;

        let arguments = match op_code.instruction_length() {
            Fixed(arguments_len) => {
                if index + 1 + arguments_len > raw_code.len() {
                    return Err(ClassReaderError::InvalidClassData(format!(
                        "cannot read arguments of instruction {op_code}"
                    )));
                }
                &raw_code[index + 1..index + 1 + arguments_len]
            }
            InstructionLength::Variable => return Err(UnsupportedInstruction(op_code)),
        };

        Ok(Self { op_code, arguments })
    }

    pub fn parse_instructions(raw_code: &'a [u8]) -> Result<Vec<Self>, ClassReaderError> {
        let mut reader = Buffer::new(raw_code);
        let mut instructions: Vec<Instruction> = Vec::new();

        while reader.has_more_data() {
            let op_byte = reader.read_u8()?;
            let op_code = OpCode::try_from(op_byte).map_err(|_| {
                ClassReaderError::InvalidClassData(format!("invalid op code: {op_byte:#04x}"))
            })?;
            let arguments = match op_code.instruction_length() {
                Fixed(arguments_len) => reader.read_bytes(arguments_len).map_err(|_| {
                    ClassReaderError::InvalidClassData(format!(
                        "cannot find arguments for instruction {:#04x}",
                        op_code as u8
                    ))
                }),
                InstructionLength::Variable => Err(UnsupportedInstruction(op_code)),
            }?;

            instructions.push(Instruction { op_code, arguments });
        }

        Ok(instructions)
    }

    pub fn argument(&self, index: usize) -> Result<u8, ClassReaderError> {
        self.arguments
            .get(index)
            .ok_or(ClassReaderError::ValidationError(
                "invalid arguments of instruction".to_string(),
            ))
            .map(|byte_ref| *byte_ref)
    }

    pub fn arguments_u16(&self, index: usize) -> Result<u16, ClassReaderError> {
        let index_byte_1 = self.argument(index)? as u16;
        let index_byte_2 = self.argument(index + 1)? as u16;
        Ok((index_byte_1 << 8) | index_byte_2)
    }

    pub fn argument_signed(&self, index: usize) -> Result<i8, ClassReaderError> {
        self.arguments
            .get(index)
            .ok_or(ClassReaderError::ValidationError(
                "invalid arguments of instruction".to_string(),
            ))
            .map(|byte_ref| unsafe { std::mem::transmute(*byte_ref) })
    }

    pub fn length(&self) -> usize {
        1 + self.arguments.len()
    }
}
