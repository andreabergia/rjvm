use std::{fmt, fmt::Formatter, str::Chars};

use itertools::Itertools;

use ClassReaderError::InvalidTypeDescriptor;

use crate::class_reader_error::ClassReaderError;

/// Models the type of one field, or one parameter of a method
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    /// Primitive types
    Base(BaseType),

    /// Standard object
    Object(String),

    /// Array
    Array(Box<FieldType>),
}

impl fmt::Display for FieldType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            FieldType::Base(base) => write!(f, "{base}"),
            FieldType::Object(class) => f.write_str(class),
            FieldType::Array(component_type) => write!(f, "{component_type}[]"),
        }
    }
}

/// Possible primitive types
#[derive(Debug, Clone, PartialEq, strum_macros::Display)]
#[repr(u8)]
pub enum BaseType {
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
    /// Parses a type descriptor as specified in the JVM specs:
    /// https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.3.2
    pub fn parse(type_descriptor: &str) -> Result<FieldType, ClassReaderError> {
        let mut chars = type_descriptor.chars();
        let descriptor = Self::parse_from(type_descriptor, &mut chars)?;
        match chars.next() {
            None => Ok(descriptor),
            Some(_) => Err(InvalidTypeDescriptor(type_descriptor.to_string())),
        }
    }

    pub(crate) fn parse_from(
        type_descriptor: &str,
        chars: &mut Chars,
    ) -> Result<FieldType, ClassReaderError> {
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
                match chars.next() {
                    Some(';') => FieldType::Object(class_name),
                    _ => return Err(InvalidTypeDescriptor(type_descriptor.to_string())),
                }
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
    use crate::{
        class_reader_error::ClassReaderError,
        field_type::{BaseType, FieldType},
    };

    #[test]
    fn cannot_parse_empty_descriptor() {
        assert!(matches!(
            FieldType::parse(""),
            Err(ClassReaderError::InvalidTypeDescriptor(s)) if s.is_empty()
        ));
    }

    #[test]
    fn cannot_parse_invalid_primitive() {
        assert!(matches!(
            FieldType::parse("W"),
            Err(ClassReaderError::InvalidTypeDescriptor(s)) if s == "W"
        ));
    }

    #[test]
    fn cannot_parse_missing_semicolon() {
        assert!(matches!(
            FieldType::parse("Ljava/lang/String"),
            Err(ClassReaderError::InvalidTypeDescriptor(s)) if s == "Ljava/lang/String"
        ));
    }

    #[test]
    fn cannot_parse_invalid_array() {
        assert!(matches!(
            FieldType::parse("["),
            Err(ClassReaderError::InvalidTypeDescriptor(s)) if s == "["
        ));
    }

    #[test]
    fn can_parse_primitive_descriptors() {
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

    #[test]
    fn can_format_base_type() {
        assert_eq!("Long", format!("{}", FieldType::parse("J").unwrap()));
    }

    #[test]
    fn can_format_object() {
        assert_eq!(
            "java/lang/String",
            format!("{}", FieldType::parse("Ljava/lang/String;").unwrap())
        );
    }

    #[test]
    fn can_format_array() {
        assert_eq!("Int[]", format!("{}", FieldType::parse("[I").unwrap()));
    }
}
