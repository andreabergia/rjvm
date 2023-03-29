use std::io::Error;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq)]
pub struct ClassLoadingError {}

impl From<Error> for ClassLoadingError {
    fn from(_: Error) -> Self {
        Self {}
    }
}

pub struct ClassPath {
    entries: Vec<Box<dyn ClassPathEntry>>,
}

impl ClassPath {
    pub fn push(&mut self, entry: Box<dyn ClassPathEntry>) {
        self.entries.push(entry)
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::class_path::{ClassPathEntry, FileSystemClassPathEntry};

    #[test]
    fn file_system_class_path_entry_works() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/resources");
        let entry = FileSystemClassPathEntry::new(path);

        let buf = entry
            .resolve("rjvm/ControlFlow")
            .expect("should have been able to read file")
            .expect("should have been able to find file");
        let magic_number =
            u32::from_be_bytes(buf[0..4].try_into().expect("file should have 4 bytes"));
        assert_eq!(0xCAFEBABE, magic_number);

        assert!(entry
            .resolve("rjvm/Foo")
            .expect("should not have had any errors")
            .is_none());
    }
}
