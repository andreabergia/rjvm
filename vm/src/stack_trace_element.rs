use rjvm_reader::line_number::LineNumber;
use std::{
    fmt,
    fmt::{Display, Formatter},
};

/// One element of the stack trace information. Models java.lang.StackTraceElement
#[derive(Debug, Clone)]
pub struct StackTraceElement<'a> {
    pub class_name: &'a str,
    pub method_name: &'a str,
    pub source_file: &'a Option<String>,
    pub line_number: Option<LineNumber>,
}

impl<'a> Display for StackTraceElement<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(file_name) = self.source_file {
            if let Some(line_number) = self.line_number {
                write!(
                    f,
                    "{}::{} ({}:{})",
                    self.class_name, self.method_name, file_name, line_number
                )
            } else {
                write!(
                    f,
                    "{}::{} ({})",
                    self.class_name, self.method_name, file_name
                )
            }
        } else {
            write!(f, "{}::{}", self.class_name, self.method_name)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::stack_trace_element::StackTraceElement;
    use rjvm_reader::line_number::LineNumber;

    #[test]
    fn can_format_without_source_file_or_line_number() {
        let element = StackTraceElement {
            class_name: "Object",
            method_name: "<init>",
            source_file: &None,
            line_number: None,
        };
        assert_eq!("Object::<init>", format!("{element}"));
    }

    #[test]
    fn can_format_with_source_file_but_no_line_number() {
        let element = StackTraceElement {
            class_name: "Object",
            method_name: "<init>",
            source_file: &Some("Object.java".to_string()),
            line_number: None,
        };
        assert_eq!("Object::<init> (Object.java)", format!("{element}"));
    }

    #[test]
    fn can_format_with_source_file_and_line_number() {
        let element = StackTraceElement {
            class_name: "Object",
            method_name: "<init>",
            source_file: &Some("Object.java".to_string()),
            line_number: Some(LineNumber(42)),
        };
        assert_eq!("Object::<init> (Object.java:42)", format!("{element}"));
    }
}
