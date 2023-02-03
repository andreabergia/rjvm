use std::{fs::File, io::Read, path::Path};

use crate::attribute::Attribute;
use crate::class_file_field::ClassFileField;
use crate::field_flags::FieldFlags;
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
        self.read_fields()?;

        Ok(self.class_file)
    }

    fn check_magic_number(&mut self) -> Result<()> {
        match self.buffer.read_u32() {
            Ok(0xCAFEBABE) => Ok(()),
            Ok(_) => Err(ClassReaderError::InvalidClassData(
                "invalid magic number".to_owned(),
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
            self.read_string_reference(super_constant_idx)
        }
    }

    fn read_string_reference(&self, index: u16) -> Result<String> {
        self.class_file
            .constants
            .text_of(index)
            .map_err(|err| err.into())
    }

    fn read_interfaces(&mut self) -> Result<()> {
        let interfaces_count = self.buffer.read_u16()?;
        self.class_file.interfaces = (0..interfaces_count)
            .map(|_| self.read_class_reference())
            .collect::<Result<Vec<String>>>()?;
        Ok(())
    }

    fn read_fields(&mut self) -> Result<()> {
        let fields_count = self.buffer.read_u16()?;
        self.class_file.fields = (0..fields_count)
            .map(|_| self.read_field())
            .collect::<Result<Vec<ClassFileField>>>()?;
        Ok(())
    }
    fn read_field(&mut self) -> Result<ClassFileField> {
        let flags = self.read_field_flags()?;
        let name_constant_index = self.buffer.read_u16()?;
        let name = self.read_string_reference(name_constant_index)?;
        let type_constant_index = self.buffer.read_u16()?;
        let type_descriptor = self.read_string_reference(type_constant_index)?;
        let attributes = self.read_attributes()?;

        Ok(ClassFileField {
            flags,
            name,
            type_descriptor,
            attributes,
        })
    }

    fn read_field_flags(&mut self) -> Result<FieldFlags> {
        let field_flags_bits = self.buffer.read_u16()?;
        match FieldFlags::from_bits(field_flags_bits) {
            Some(bitset) => Ok(bitset),
            None => Err(ClassReaderError::InvalidClassData(format!(
                "invalid class flags: {}",
                field_flags_bits
            ))),
        }
    }

    fn read_attributes(&mut self) -> Result<Vec<Attribute>> {
        let attributes_count = self.buffer.read_u16()?;
        (0..attributes_count)
            .map(|_| self.read_attribute())
            .collect::<Result<Vec<Attribute>>>()
    }

    fn read_attribute(&mut self) -> Result<Attribute> {
        let name_constant_index = self.buffer.read_u16()?;
        let name = self.read_string_reference(name_constant_index)?;
        let len = self.buffer.read_u32()?;
        let bytes = self
            .buffer
            .read_bytes(usize::try_from(len).expect("usize should have at least 32 bits"))?;
        Ok(Attribute {
            name,
            info: Vec::from(bytes),
        })
    }
}

pub fn read(path: &Path) -> Result<ClassFile> {
    println!("Reading class from file {}", path.display());

    let mut file = File::open(path)?;
    let mut buf: Vec<u8> = Vec::new();
    file.read_to_end(&mut buf)?;

    read_buffer(&buf)
}

pub fn read_buffer(buf: &[u8]) -> Result<ClassFile> {
    Parser::new(buf).parse()
}

#[cfg(test)]
mod tests {
    use crate::class_reader::read_buffer;
    use crate::class_reader_error::ClassReaderError;

    #[test]
    fn magic_number_is_required() {
        let data = vec![0x00, 0x01, 0x02, 0x03];
        assert!(matches!(
            read_buffer(&data),
            Err(ClassReaderError::InvalidClassData(s)) if s == "invalid magic number"
        ));
    }
}
