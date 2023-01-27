use crate::class_reader_error::{ClassReaderError, Result};

pub struct Buffer<'a> {
    buffer: &'a [u8],
    position: usize,
}

impl<'a> Buffer<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Buffer {
            buffer: data,
            position: 0,
        }
    }

    fn advance(&mut self, size: usize) -> Result<&[u8]> {
        if self.position + size > self.buffer.len() {
            Err(ClassReaderError::InvalidClassData(
                "class does not have expected length".to_string(),
            ))
        } else {
            let slice = &self.buffer[self.position..self.position + size];
            self.position += size;
            Ok(slice)
        }
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        self.advance(std::mem::size_of::<u32>())
            .map(|bytes| u32::from_be_bytes(bytes.try_into().unwrap()))
    }

    pub fn has_more_data(&self) -> bool {
        self.position < self.buffer.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::buffer::Buffer;

    #[test]
    fn buffer_works() {
        let data = vec![0x00, 0x00, 0x00, 0x42];
        let mut buffer = Buffer::new(&data);

        assert_eq!(true, buffer.has_more_data());
        assert_eq!(0x42u32, buffer.read_u32().unwrap());
        assert_eq!(false, buffer.has_more_data());

        assert_eq!(true, buffer.read_u32().is_err());
    }
}
