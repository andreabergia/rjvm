use std::{
    cell::RefCell,
    fs::File,
    io::{Error, Read},
    path::{Path, PathBuf},
};

use thiserror::Error;
use zip::{result::ZipError, ZipArchive};

use crate::class_path_entry::JarFileError::{InvalidJar, NotFound, ReadingError};

#[derive(Debug, PartialEq)]
pub struct ClassLoadingError {}

impl From<Error> for ClassLoadingError {
    fn from(_: Error) -> Self {
        Self {}
    }
}

pub trait ClassPathEntry {
    // TODO: should `class_name` be a newtype?
    fn resolve(&self, class_name: &str) -> Result<Option<Vec<u8>>, ClassLoadingError>;
}

/// Implementation of [ClassPathEntry] that searches for `.class` files,
/// using the given directory as the root package
#[derive(Debug)]
pub struct FileSystemClassPathEntry {
    base_directory: PathBuf,
}

impl FileSystemClassPathEntry {
    #[allow(dead_code)]
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let mut base_directory = PathBuf::new();
        base_directory.push(path);
        Self { base_directory }
    }
}

impl ClassPathEntry for FileSystemClassPathEntry {
    fn resolve(&self, class_name: &str) -> Result<Option<Vec<u8>>, ClassLoadingError> {
        let mut candidate = self.base_directory.clone();
        candidate.push(class_name);
        candidate.set_extension("class");
        if candidate.exists() {
            std::fs::read(candidate).map(Some).map_err(|err| err.into())
        } else {
            Ok(None)
        }
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum JarFileError {
    #[error("file not found")]
    NotFound,
    #[error("error reading file")]
    ReadingError,
    #[error("file is not a valid jar")]
    InvalidJar,
}

/// Implementation of [ClassPathEntry] that searches for `.class` file inside a `.jar` file
#[derive(Debug)]
pub struct JarFileClassPathEntry {
    zip: RefCell<ZipArchive<File>>,
}

impl JarFileClassPathEntry {
    #[allow(dead_code)]
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, JarFileError> {
        if !path.as_ref().exists() {
            return Err(NotFound);
        }
        let file = File::open(path).map_err(|_| ReadingError)?;
        let zip = ZipArchive::new(file).map_err(|_| InvalidJar)?;
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
                    .map_err(|_| ClassLoadingError {})?;
                Ok(Some(buffer))
            }
            Err(err) => match err {
                ZipError::FileNotFound => Ok(None),
                _ => Err(ClassLoadingError {}),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::class_path::{
        ClassPathEntry, FileSystemClassPathEntry, JarFileClassPathEntry, JarFileError,
    };

    #[test]
    fn file_system_class_path_entry_works() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/resources");
        let entry = FileSystemClassPathEntry::new(path);

        assert_can_find_class(&entry, "rjvm/NumericTypes");
        assert_can_find_class(&entry, "rjvm/ControlFlow");
        assert_cannot_find_class(&entry, "rjvm/Foo");
    }

    #[test]
    fn jar_file_not_found() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/resources/not_found.jar");
        let entry = JarFileClassPathEntry::new(path);
        assert_eq!(
            JarFileError::NotFound,
            entry.expect_err("should have thrown an error")
        );
    }

    #[test]
    fn file_is_not_a_jar() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/resources/compile.sh");
        let entry = JarFileClassPathEntry::new(path);
        assert_eq!(
            JarFileError::InvalidJar,
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

    fn assert_can_find_class(entry: &impl ClassPathEntry, class_name: &str) {
        let buf = entry
            .resolve(class_name)
            .expect("should have been able to read file")
            .expect("should have been able to find file");
        let magic_number =
            u32::from_be_bytes(buf[0..4].try_into().expect("file should have 4 bytes"));
        assert_eq!(0xCAFEBABE, magic_number);
    }

    fn assert_cannot_find_class(entry: &impl ClassPathEntry, class_name: &str) {
        assert!(entry
            .resolve(class_name)
            .expect("should not have had any errors")
            .is_none());
    }
}
