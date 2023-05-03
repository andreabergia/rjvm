use log::debug;
use thiserror::Error;

use crate::{
    class_path_entry::{ClassLoadingError, ClassPathEntry},
    file_system_class_path_entry::FileSystemClassPathEntry,
    jar_file_class_path_entry::JarFileClassPathEntry,
};

#[allow(dead_code)]
#[derive(Default, Debug)]
pub struct ClassPath {
    entries: Vec<Box<dyn ClassPathEntry>>,
}

#[derive(Error, Debug, PartialEq)]
pub enum ClassPathParseError {
    #[error("invalid classpath entry: {0}")]
    InvalidEntry(String),
}

impl ClassPath {
    pub fn push(&mut self, string: &str) -> Result<(), ClassPathParseError> {
        let mut entries_to_add: Vec<Box<dyn ClassPathEntry>> = Vec::new();
        for entry in string.split(':') {
            debug!("trying to parse class path entry {}", entry);
            let parsed_entry = Self::try_parse_entry(entry)?;
            entries_to_add.push(parsed_entry);
        }
        self.entries.append(&mut entries_to_add);
        Ok(())
    }

    fn try_parse_entry(path: &str) -> Result<Box<dyn ClassPathEntry>, ClassPathParseError> {
        Self::try_parse_entry_as_jar(path).or_else(|_| Self::try_parse_entry_as_directory(path))
    }

    fn try_parse_entry_as_jar(path: &str) -> Result<Box<dyn ClassPathEntry>, ClassPathParseError> {
        let entry = JarFileClassPathEntry::new(path)
            .map_err(|_| ClassPathParseError::InvalidEntry(path.to_string()))?;
        Ok(Box::new(entry))
    }

    fn try_parse_entry_as_directory(
        path: &str,
    ) -> Result<Box<dyn ClassPathEntry>, ClassPathParseError> {
        let entry = FileSystemClassPathEntry::new(path)
            .map_err(|_| ClassPathParseError::InvalidEntry(path.to_string()))?;
        Ok(Box::new(entry))
    }

    pub fn resolve(&self, class_name: &str) -> Result<Option<Vec<u8>>, ClassLoadingError> {
        for entry in self.entries.iter() {
            debug!("looking up class {} in {:?}", class_name, entry);
            let entry_result = entry.resolve(class_name)?;
            if let Some(class_bytes) = entry_result {
                return Ok(Some(class_bytes));
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::ClassPath;

    #[test]
    fn can_parse_valid_classpath_entries() {
        let dir = env!("CARGO_MANIFEST_DIR");
        let mut class_path: ClassPath = Default::default();
        class_path
            .push(&format!(
                "{dir}/tests/resources/sample.jar:{dir}/tests/resources",
            ))
            .expect("should be able to parse classpath");
        assert_can_find_class(&class_path, "rjvm/NumericTypes"); // From jar
        assert_can_find_class(&class_path, "rjvm/SimpleMain"); // From directory
        assert_cannot_find_class(&class_path, "foo");
    }

    fn assert_can_find_class(class_path: &ClassPath, class_name: &str) {
        let buf = class_path
            .resolve(class_name)
            .expect("should not have had any errors")
            .expect("should have been able to find file");
        let magic_number =
            u32::from_be_bytes(buf[0..4].try_into().expect("file should have 4 bytes"));
        assert_eq!(0xCAFEBABE, magic_number);
    }

    fn assert_cannot_find_class(class_path: &ClassPath, class_name: &str) {
        assert!(class_path
            .resolve(class_name)
            .expect("should not have had any errors")
            .is_none());
    }
}
