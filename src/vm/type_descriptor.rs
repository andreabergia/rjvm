use ClassReaderError::InvalidTypeDescriptor;

use crate::reader::class_reader_error::ClassReaderError;
use crate::vm::type_descriptor::FieldType::Base;

#[derive(Debug, Clone, PartialEq)]
enum FieldType {
    Base(BaseType),
    Object(String),
    Array(Box<FieldType>),
}

#[derive(Debug, Clone, PartialEq)]
enum BaseType {
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Short,
    Boolean,
}

impl FieldType {
    fn parse(type_descriptor: &str) -> Result<FieldType, ClassReaderError> {
        let mut chars = type_descriptor.chars();
        let first_char = chars
            .next()
            .ok_or(InvalidTypeDescriptor(type_descriptor.to_string()))?;

        let descriptor = match first_char {
            'B' => Base(BaseType::Byte),
            'C' => Base(BaseType::Char),
            'D' => Base(BaseType::Double),
            'F' => Base(BaseType::Float),
            'I' => Base(BaseType::Int),
            'J' => Base(BaseType::Long),
            'S' => Base(BaseType::Short),
            'Z' => Base(BaseType::Boolean),
            _ => return Err(InvalidTypeDescriptor(type_descriptor.to_string())),
        };
        if let Some(_) = chars.next() {
            Err(InvalidTypeDescriptor(type_descriptor.to_string()))
        } else {
            Ok(descriptor)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::reader::class_reader_error::ClassReaderError;
    use crate::vm::type_descriptor::{BaseType, FieldType};

    #[test]
    fn cannot_parse_empty_descriptor() {
        assert!(matches!(
            FieldType::parse(""),
            Err(ClassReaderError::InvalidTypeDescriptor(s)) if s.is_empty()
        ));
    }

    #[test]
    fn can_parse_simple_descriptors() {
        assert_eq!(Ok(FieldType::Base(BaseType::Byte)), FieldType::parse("B"));
        assert_eq!(Ok(FieldType::Base(BaseType::Char)), FieldType::parse("C"));
        assert_eq!(Ok(FieldType::Base(BaseType::Double)), FieldType::parse("D"));
        assert_eq!(Ok(FieldType::Base(BaseType::Float)), FieldType::parse("F"));
        assert_eq!(Ok(FieldType::Base(BaseType::Int)), FieldType::parse("I"));
        assert_eq!(Ok(FieldType::Base(BaseType::Long)), FieldType::parse("J"));
        assert_eq!(Ok(FieldType::Base(BaseType::Short)), FieldType::parse("S"));
        assert_eq!(
            Ok(FieldType::Base(BaseType::Boolean)),
            FieldType::parse("Z")
        );
    }
}
