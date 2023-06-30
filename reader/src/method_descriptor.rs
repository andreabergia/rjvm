use std::{fmt, fmt::Formatter, str::Chars};

use itertools::Itertools;

use crate::{
    class_reader_error::{ClassReaderError, ClassReaderError::InvalidTypeDescriptor},
    field_type::FieldType,
};

/// Models the signature of a method, i.e. the type of the parameters it takes and the type
/// of the return value
#[derive(Debug, Default, Clone, PartialEq)]
pub struct MethodDescriptor {
    pub parameters: Vec<FieldType>,
    pub return_type: Option<FieldType>,
}

impl fmt::Display for MethodDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("(")?;
        f.write_str(&self.parameters.iter().join(", "))?;
        match &self.return_type {
            Some(field_type) => write!(f, ") -> {field_type}"),
            None => f.write_str(") -> void"),
        }
    }
}

impl MethodDescriptor {
    /// Parses a method descriptor as specified in the JVM specs:
    /// https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.3.3
    pub fn parse(descriptor: &str) -> Result<MethodDescriptor, ClassReaderError> {
        let mut chars = descriptor.chars();
        match chars.next() {
            Some('(') => {
                let parameters = Self::parse_parameters(descriptor, &mut chars)?;
                if Some(')') == chars.next() {
                    let return_type = Self::parse_return_type(descriptor, &mut chars)?;
                    Ok(MethodDescriptor {
                        parameters,
                        return_type,
                    })
                } else {
                    Err(InvalidTypeDescriptor(descriptor.to_string()))
                }
            }
            _ => Err(InvalidTypeDescriptor(descriptor.to_string())),
        }
    }

    fn parse_parameters(
        descriptor: &str,
        chars: &mut Chars,
    ) -> Result<Vec<FieldType>, ClassReaderError> {
        let mut parameters = Vec::new();
        loop {
            match chars.clone().next() {
                Some(')') => return Ok(parameters),
                Some(_) => {
                    let param = FieldType::parse_from(descriptor, chars)?;
                    parameters.push(param);
                }
                None => return Err(InvalidTypeDescriptor(descriptor.to_string())),
            }
        }
    }

    fn parse_return_type(
        descriptor: &str,
        chars: &mut Chars,
    ) -> Result<Option<FieldType>, ClassReaderError> {
        match chars.clone().next() {
            Some('V') => Ok(None),
            Some(_) => {
                let return_type = Some(FieldType::parse_from(descriptor, chars)?);
                if chars.next().is_none() {
                    Ok(return_type)
                } else {
                    Err(InvalidTypeDescriptor(descriptor.to_string()))
                }
            }
            _ => Err(InvalidTypeDescriptor(descriptor.to_string())),
        }
    }

    pub fn num_arguments(&self) -> usize {
        self.parameters.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        class_reader_error::ClassReaderError,
        field_type::{BaseType, FieldType},
        method_descriptor::MethodDescriptor,
    };

    #[test]
    fn cannot_parse_empty_descriptor() {
        assert_cannot_parse("")
    }

    #[test]
    fn cannot_parse_invalid_descriptor_no_arguments() {
        assert_cannot_parse("J")
    }

    #[test]
    fn cannot_parse_invalid_descriptor_no_return_type() {
        assert_cannot_parse("(J)")
    }

    #[test]
    fn cannot_parse_invalid_descriptor_trash_after() {
        assert_cannot_parse("()JJ")
    }

    fn assert_cannot_parse(descriptor: &str) {
        assert!(matches!(
            MethodDescriptor::parse(descriptor),
            Err(ClassReaderError::InvalidTypeDescriptor(s)) if s == descriptor
        ));
    }

    #[test]
    fn can_parse_primitives() {
        assert_eq!(
            Ok(MethodDescriptor {
                parameters: vec![
                    FieldType::Base(BaseType::Long),
                    FieldType::Base(BaseType::Int)
                ],
                return_type: Some(FieldType::Base(BaseType::Double)),
            }),
            MethodDescriptor::parse("(JI)D"),
        );
    }

    #[test]
    fn can_parse_no_args_void_return() {
        assert_eq!(
            Ok(MethodDescriptor {
                parameters: vec![],
                return_type: None,
            }),
            MethodDescriptor::parse("()V"),
        );
    }

    #[test]
    fn can_parse_arrays_objects() {
        assert_eq!(
            Ok(MethodDescriptor {
                parameters: vec![
                    FieldType::Object("java/lang/String".to_string()),
                    FieldType::Base(BaseType::Int),
                ],
                return_type: Some(FieldType::Array(Box::new(FieldType::Base(BaseType::Long)))),
            }),
            MethodDescriptor::parse("(Ljava/lang/String;I)[J"),
        );
    }

    #[test]
    fn can_format_void_to_void() {
        assert_eq!(
            "() -> void",
            format!("{}", MethodDescriptor::parse("()V").unwrap())
        );
    }

    #[test]
    fn can_format_parameters_to_return_type() {
        assert_eq!(
            "(java/lang/String, Int) -> Long[]",
            format!(
                "{}",
                MethodDescriptor::parse("(Ljava/lang/String;I)[J").unwrap()
            )
        );
    }

    #[test]
    fn can_get_num_arguments() {
        assert_eq!(
            2,
            MethodDescriptor::parse("(Ljava/lang/String;I)[J")
                .unwrap()
                .num_arguments(),
        );
    }
}
