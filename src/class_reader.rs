use std::{fs::File, io::Read, path::Path};

use crate::{
    buffer::Buffer,
    class_access_flags::ClassAccessFlags,
    class_file::ClassFile,
    class_file_version::ClassFileVersion,
    class_reader_error::{ClassReaderError, Result},
    constant_pool::ConstantPoolEntry,
};

struct Parser<'a> {
    buffer: Buffer<'a>,
    class_file: ClassFile,
}

impl<'a> Parser<'a> {
    fn new(data: &[u8]) -> Parser {
        Parser {
            buffer: Buffer::new(data),
            class_file: Default::default(),
        }
    }

    fn parse(mut self) -> Result<ClassFile> {
        self.check_magic_number()?;
        self.read_version()?;
        self.read_constants()?;
        self.read_access_flags()?;
        self.class_file.name = self.read_class_reference()?;
        self.class_file.superclass = self.read_class_reference()?;
        self.read_interfaces()?;

        Ok(self.class_file)
    }

    fn check_magic_number(&mut self) -> Result<()> {
        match self.buffer.read_u32() {
            Ok(0xCAFEBABE) => Ok(()),
            Ok(_) => Err(ClassReaderError::InvalidClassData(
                "Invalid magic number".to_owned(),
            )),
            Err(err) => Err(err),
        }
    }

    fn read_version(&mut self) -> Result<()> {
        let minor_version = self.buffer.read_u16()?;
        let major_version = self.buffer.read_u16()?;

        self.class_file.version = ClassFileVersion::from(major_version, minor_version)?;
        Ok(())
    }

    fn read_constants(&mut self) -> Result<()> {
        let num = self.buffer.read_u16()?;
        for i in 0..num - 1 {
            let tag = self.buffer.read_u8()?;
            let constant = match tag {
                1 => self.parse_string_constant()?,
                3 => self.parse_int_constant()?,
                4 => self.parse_float_constant()?,
                5 => self.parse_long_constant()?,
                6 => self.parse_double_constant()?,
                7 => self.parse_class_reference_constant()?,
                8 => self.parse_string_reference_constant()?,
                9 => self.parse_field_reference_constant()?,
                10 => self.parse_method_reference_constant()?,
                11 => self.parse_interface_method_reference_constant()?,
                12 => self.parse_name_and_type_constant()?,
                _ => {
                    println!("Constant {} is of type {}", i, tag);
                    return Err(ClassReaderError::InvalidClassData(format!(
                        "Unknown constant type: 0x{:X}",
                        tag
                    )));
                }
            };
            self.class_file.constants.add(constant);
        }

        Ok(())
    }

    fn parse_string_constant(&mut self) -> Result<ConstantPoolEntry> {
        let len = self.buffer.read_u16()?;
        self.buffer
            .read_utf8(len as usize)
            .map(ConstantPoolEntry::String)
    }

    fn parse_int_constant(&mut self) -> Result<ConstantPoolEntry> {
        self.buffer.read_i32().map(ConstantPoolEntry::Integer)
    }

    fn parse_float_constant(&mut self) -> Result<ConstantPoolEntry> {
        self.buffer.read_f32().map(ConstantPoolEntry::Float)
    }

    fn parse_long_constant(&mut self) -> Result<ConstantPoolEntry> {
        self.buffer.read_i64().map(ConstantPoolEntry::Long)
    }

    fn parse_double_constant(&mut self) -> Result<ConstantPoolEntry> {
        self.buffer.read_f64().map(ConstantPoolEntry::Double)
    }

    fn parse_class_reference_constant(&mut self) -> Result<ConstantPoolEntry> {
        let fqn_string_index = self.buffer.read_u16()?;
        Ok(ConstantPoolEntry::ClassReference(fqn_string_index))
    }

    fn parse_string_reference_constant(&mut self) -> Result<ConstantPoolEntry> {
        let string_index = self.buffer.read_u16()?;
        Ok(ConstantPoolEntry::StringReference(string_index))
    }

    fn parse_method_reference_constant(&mut self) -> Result<ConstantPoolEntry> {
        let class_reference = self.buffer.read_u16()?;
        let name_and_type = self.buffer.read_u16()?;
        Ok(ConstantPoolEntry::MethodReference(
            class_reference,
            name_and_type,
        ))
    }

    fn parse_interface_method_reference_constant(&mut self) -> Result<ConstantPoolEntry> {
        let class_reference = self.buffer.read_u16()?;
        let name_and_type = self.buffer.read_u16()?;
        Ok(ConstantPoolEntry::InterfaceMethodReference(
            class_reference,
            name_and_type,
        ))
    }

    fn parse_field_reference_constant(&mut self) -> Result<ConstantPoolEntry> {
        let class_reference = self.buffer.read_u16()?;
        let name_and_type = self.buffer.read_u16()?;
        Ok(ConstantPoolEntry::FieldReference(
            class_reference,
            name_and_type,
        ))
    }

    fn parse_name_and_type_constant(&mut self) -> Result<ConstantPoolEntry> {
        let name = self.buffer.read_u16()?;
        let type_descriptor = self.buffer.read_u16()?;
        Ok(ConstantPoolEntry::NameAndTypeDescriptor(
            name,
            type_descriptor,
        ))
    }

    fn read_access_flags(&mut self) -> Result<()> {
        let num = self.buffer.read_u16()?;
        match ClassAccessFlags::from_bits(num) {
            Some(bitset) => {
                self.class_file.flags = bitset;
                Ok(())
            }
            None => Err(ClassReaderError::InvalidClassData(format!(
                "invalid class flags: {}",
                num
            ))),
        }
    }

    fn read_class_reference(&mut self) -> Result<String> {
        let super_constant_idx = self.buffer.read_u16()?;
        if super_constant_idx == 0 {
            Ok(String::from(""))
        } else {
            self.class_file
                .constants
                .text_of(super_constant_idx)
                .map_err(|err| err.into())
        }
    }

    fn read_interfaces(&mut self) -> Result<()> {
        let interfaces_count = self.buffer.read_u16()?;
        self.class_file.interfaces = (0..interfaces_count)
            .map(|_| self.read_class_reference())
            .collect::<Result<Vec<String>>>()?;
        Ok(())
    }
}

pub fn read(path: &Path) -> Result<ClassFile> {
    println!("Reading class from file {}", path.display());

    let mut file = File::open(path)?;
    let mut buf: Vec<u8> = Vec::new();
    file.read_to_end(&mut buf)?;

    Parser::new(&buf).parse()
}

// #[cfg(test)]
// mod tests {
//     use std::io;

//     use crate::class;

//     #[test]
//     fn magic_number_is_required() {
//         let data = vec![0x00];
//         let class_file = class::read(&data).map_err(|e| e.kind());

//         let expected = Err(io::ErrorKind::InvalidData);
//         assert_eq!(expected, class_file);
//     }
// }
