use std::{
    cell::RefCell,
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use thiserror::Error;
use zip::{result::ZipError, ZipArchive};

use crate::class_path_entry::{ClassLoadingError, ClassPathEntry};

/// Implementation of [ClassPathEntry] that searches for `.class` file inside a `.jar` file
#[derive(Debug)]
pub struct JarFileClassPathEntry {
    zip: RefCell<ZipArchive<BufReader<File>>>,
}

impl JarFileClassPathEntry {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, JarFileError> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(JarFileError::NotFound(path.to_string_lossy().to_string()));
        }
        let file = File::open(path)
            .map_err(|_| JarFileError::ReadingError(path.to_string_lossy().to_string()))?;
        let buf_reader = BufReader::new(file);
        let zip = ZipArchive::new(buf_reader)
            .map_err(|_| JarFileError::InvalidJar(path.to_string_lossy().to_string()))?;
        Ok(Self {
            zip: RefCell::new(zip),
        })
    }
}

impl ClassPathEntry for JarFileClassPathEntry {
    fn resolve(&self, class_name: &str) -> Result<Option<Vec<u8>>, ClassLoadingError> {
        let class_file_name = class_name.to_string() + ".class";
        match self.zip.borrow_mut().by_name(&class_file_name) {
            Ok(mut zip_file) => {
                let mut buffer: Vec<u8> = Vec::with_capacity(zip_file.size() as usize);
                zip_file
                    .read_to_end(&mut buffer)
                    .map_err(ClassLoadingError::new)?;
                Ok(Some(buffer))
            }
            Err(err) => match err {
                ZipError::FileNotFound => Ok(None),
                _ => Err(ClassLoadingError::new(err)),
            },
        }
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum JarFileError {
    #[error("file {0} not found")]
    NotFound(String),
    #[error("error reading file {0}")]
    ReadingError(String),
    #[error("file {0} is not a valid jar")]
    InvalidJar(String),
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        class_path_entry::tests::{assert_can_find_class, assert_cannot_find_class},
        jar_file_class_path_entry::{JarFileClassPathEntry, JarFileError},
    };

    #[test]
    fn jar_file_not_found() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/resources/not_found.jar");
        let entry = JarFileClassPathEntry::new(path.clone());
        assert_eq!(
            JarFileError::NotFound(path.to_string_lossy().to_string()),
            entry.expect_err("should have thrown an error")
        );
    }

    #[test]
    fn file_is_not_a_jar() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/resources/compile.sh");

        let entry = JarFileClassPathEntry::new(path.clone());
        assert_eq!(
            JarFileError::InvalidJar(path.to_string_lossy().to_string()),
            entry.expect_err("should have thrown an error")
        );
    }

    #[test]
    fn valid_jar_file_can_search_for_class_file() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/resources/sample.jar");
        let entry = JarFileClassPathEntry::new(path).expect("should have read the jar file");

        assert_can_find_class(&entry, "rjvm/NumericTypes");
        assert_can_find_class(&entry, "rjvm/ControlFlow");
        assert_cannot_find_class(&entry, "rjvm/Foo");
    }
}
