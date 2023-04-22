use std::fmt;
use std::io::Error;

#[derive(Debug, PartialEq)]
pub struct ClassLoadingError {}

// TODO
impl From<Error> for ClassLoadingError {
    fn from(_: Error) -> Self {
        Self {}
    }
}

pub trait ClassPathEntry: fmt::Debug {
    // TODO: should `class_name` be a newtype?
    fn resolve(&self, class_name: &str) -> Result<Option<Vec<u8>>, ClassLoadingError>;
}

#[cfg(test)]
pub mod tests {
    use crate::class_path_entry::ClassPathEntry;

    pub fn assert_can_find_class(entry: &impl ClassPathEntry, class_name: &str) {
        let buf = entry
            .resolve(class_name)
            .expect("should have been able to read file")
            .expect("should have been able to find file");
        let magic_number =
            u32::from_be_bytes(buf[0..4].try_into().expect("file should have 4 bytes"));
        assert_eq!(0xCAFEBABE, magic_number);
    }

    pub fn assert_cannot_find_class(entry: &impl ClassPathEntry, class_name: &str) {
        assert!(entry
            .resolve(class_name)
            .expect("should not have had any errors")
            .is_none());
    }
}
