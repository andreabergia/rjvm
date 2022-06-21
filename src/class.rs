use std::io::{Error, ErrorKind, Result};

#[derive(Debug, PartialEq)]
pub struct ClassFile {
    pub name: String,
}

pub type ClassResult = Result<ClassFile>;

struct BufferReader<'a> {
    data: &'a [u8],
}

const SIZE_U32: usize = std::mem::size_of::<u32>();

impl<'a> BufferReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        BufferReader { data }
    }

    fn next_u32(&mut self) -> Result<u32> {
        if self.data.len() < SIZE_U32 {
            Err(Error::new(ErrorKind::InvalidData, "Not enough data"))
        } else {
            let (num_slice, rest) = self.data.split_at(SIZE_U32);
            let read = u32::from_be_bytes(num_slice.try_into().unwrap());
            self.data = rest;
            Ok(read)
        }
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
