use std::fmt;
use std::fmt::Formatter;

use crate::reader::class_reader_error::ClassReaderError;
use crate::reader::class_reader_error::ClassReaderError::UnsupportedInstruction;
use crate::reader::opcodes::InstructionLength::Fixed;
use crate::reader::opcodes::{InstructionLength, OpCode};
use crate::utils::buffer::Buffer;

#[derive(Debug, PartialEq)]
pub struct Instruction {
    pub op_code: OpCode,
    pub arguments: Vec<u8>,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.arguments.is_empty() {
            write!(f, "{}", self.op_code)
        } else {
            write!(f, "{} {:?}", self.op_code, self.arguments)
        }
    }
}

impl Instruction {
    pub fn parse_instructions(raw_code: &[u8]) -> Result<Vec<Self>, ClassReaderError> {
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

            instructions.push(Instruction {
                op_code,
                arguments: Vec::from(arguments),
            });
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
}
