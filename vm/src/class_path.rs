use thiserror::Error;

use crate::class_path_entry::{
    ClassLoadingError, ClassPathEntry, FileSystemClassPathEntry, JarFileClassPathEntry,
};

#[allow(dead_code)]
#[derive(Default)]
pub struct ClassPath {
    entries: Vec<Box<dyn ClassPathEntry>>,
}

#[derive(Error, Debug, PartialEq)]
pub enum ClassPathParseError {
    #[error("invalid entry: {0}")]
    InvalidEntry(String),
}

impl ClassPath {
    pub fn parse(string: &str) -> Result<ClassPath, ClassPathParseError> {
        let mut class_path: ClassPath = Default::default();
        for entry in string.split(':') {
            let parsed_entry = Self::try_parse_entry(entry)?;
            class_path.push(parsed_entry);
        }
        Ok(class_path)
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

    pub fn push(&mut self, entry: Box<dyn ClassPathEntry>) {
        self.entries.push(entry)
    }

    pub fn resolve(&self, class_name: &str) -> Result<Option<Vec<u8>>, ClassLoadingError> {
        for entry in self.entries.iter() {
            if let Ok(Some(class_bytes)) = entry.resolve(class_name) {
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
        let class_path = format!("{}/tests/resources/sample.jar:{}/tests/resources", dir, dir);
        let class_path = ClassPath::parse(&class_path).expect("should be able to parse classpath");
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
