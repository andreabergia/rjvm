use thiserror::Error;

use crate::reader::class_file_method::ClassFileMethodCode;
use crate::reader::class_reader_error::ClassReaderError;
use crate::utils::buffer::Buffer;
use crate::vm::code_printer::VmError::{InvalidData, InvalidOpCode, UnsupportedInstruction};
use crate::vm::opcodes::{InstructionLength, OpCode};

#[derive(Error, Debug)]
pub enum VmError {
    #[error("invalid data: {0}")]
    InvalidData(#[from] ClassReaderError),

    #[error("invalid Op-Code: {0}")]
    InvalidOpCode(u8),

    #[error("invalid arguments for instruction: {0}")]
    InvalidInstructionArguments(OpCode),

    #[error("unsupported instruction: {0}")]
    UnsupportedInstruction(OpCode),
}

pub fn print_code(code: &ClassFileMethodCode) -> Result<(), VmError> {
    let mut reader = Buffer::new(&code.code);

    while reader.has_more_data() {
        let op_byte = reader.read_u8()?;
        let opcode = OpCode::try_from(op_byte).map_err(|_| InvalidOpCode(op_byte))?;
        let arguments = match opcode.instruction_length() {
            InstructionLength::Fixed(arguments_len) => reader
                .read_bytes(arguments_len)
                .map_err(|_| VmError::InvalidInstructionArguments(opcode)),
            InstructionLength::Variable => Err(UnsupportedInstruction(opcode)),
        }?;
        println!("    {} {:?}", opcode, arguments);
    }
    Ok(())
}
