use std::io::{Error, ErrorKind, Result};

const SIZE_U32: usize = std::mem::size_of::<u32>();

pub struct BufferReader<'a> {
    buffer: &'a [u8],
    position: usize,
}

impl<'a> BufferReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
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

    pub fn next_u32(&mut self) -> Result<u32> {
        let num_slice = self.advance(SIZE_U32)?;
        let read = u32::from_be_bytes(num_slice.try_into().unwrap());
        Ok(read)
    }
}
