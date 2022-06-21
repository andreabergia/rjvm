use std::io::{Error, ErrorKind, Result};

#[derive(Debug, PartialEq)]
pub struct ClassFile {
    pub name: String,
}

pub type ClassResult = Result<ClassFile>;

struct BufferReader<'a> {
    buffer: &'a [u8],
    position: usize,
}

const SIZE_U32: usize = std::mem::size_of::<u32>();

impl<'a> BufferReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        BufferReader {
            buffer: data,
            position: 0,
        }
    }

    fn advance(&mut self, size: usize) -> Result<&[u8]> {
        if self.buffer.len() < size {
            Err(Error::new(ErrorKind::InvalidData, "Not enough data"))
        } else {
            let slice = &self.buffer[self.position..self.position + size];
            self.position += size;
            Ok(slice)
        }
    }

    fn next_u32(&mut self) -> Result<u32> {
        let num_slice = self.advance(SIZE_U32)?;
        let read = u32::from_be_bytes(num_slice.try_into().unwrap());
        Ok(read)
    }
}

pub fn read(data: &[u8]) -> ClassResult {
    let mut reader = BufferReader::new(data);
    let magic = reader.next_u32()?;
    if magic != 0xCAFEBABE {
        return Err(Error::new(ErrorKind::InvalidData, "No magic number"));
    }

    Ok(ClassFile {
        name: "todo".to_string(),
    })
}
