use std::str::Chars;

use itertools::Itertools;

use ClassReaderError::InvalidTypeDescriptor;

use crate::reader::class_reader_error::ClassReaderError;

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
        let descriptor = Self::parse_from(type_descriptor, &mut chars)?;
        match chars.next() {
            None => Ok(descriptor),
            Some(_) => Err(InvalidTypeDescriptor(type_descriptor.to_string())),
        }
    }

    fn parse_from(type_descriptor: &str, chars: &mut Chars) -> Result<FieldType, ClassReaderError> {
        let first_char = chars
            .next()
            .ok_or(InvalidTypeDescriptor(type_descriptor.to_string()))?;

        Ok(match first_char {
            'B' => FieldType::Base(BaseType::Byte),
            'C' => FieldType::Base(BaseType::Char),
            'D' => FieldType::Base(BaseType::Double),
            'F' => FieldType::Base(BaseType::Float),
            'I' => FieldType::Base(BaseType::Int),
            'J' => FieldType::Base(BaseType::Long),
            'S' => FieldType::Base(BaseType::Short),
            'Z' => FieldType::Base(BaseType::Boolean),
            'L' => {
                let class_name: String = chars.take_while_ref(|c| *c != ';').collect();
                chars.next(); // Consume the ;
                FieldType::Object(class_name)
            }
            '[' => {
                let component_type = Self::parse_from(type_descriptor, chars)?;
                FieldType::Array(Box::new(component_type))
            }
            _ => return Err(InvalidTypeDescriptor(type_descriptor.to_string())),
        })
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

    #[test]
    fn can_parse_object_descriptors() {
        assert_eq!(
            Ok(FieldType::Object("rjvm/Test".to_string())),
            FieldType::parse("Lrjvm/Test;")
        );
    }

    #[test]
    fn can_parse_array_description() {
        assert_eq!(
            Ok(FieldType::Array(Box::new(FieldType::Base(BaseType::Int)))),
            FieldType::parse("[I")
        );
        assert_eq!(
            Ok(FieldType::Array(Box::new(FieldType::Object(
                "java/lang/String".to_string()
            )))),
            FieldType::parse("[Ljava/lang/String;")
        );

        assert_eq!(
            Ok(FieldType::Array(Box::new(FieldType::Array(Box::new(
                FieldType::Base(BaseType::Double)
            ))))),
            FieldType::parse("[[D")
        );
    }
}
