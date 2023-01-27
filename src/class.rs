use std::io::{Error, ErrorKind, Result};

use crate::buffer_reader::BufferReader;

#[derive(Debug, PartialEq)]
pub struct ClassFile {
    pub name: String,
}

pub type ClassResult = Result<ClassFile>;

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

#[cfg(test)]
mod tests {
    use std::io;

    use crate::class;

    #[test]
    fn magic_number_is_required() {
        let data = vec![0x00];
        let class_file = class::read(&data).map_err(|e| e.kind());

        let expected = Err(io::ErrorKind::InvalidData);
        assert_eq!(expected, class_file);
    }
}
