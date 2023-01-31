use crate::class_reader_error::ClassReaderError;
use crate::class_reader_error::Result;

#[derive(Debug, PartialEq, Default, strum_macros::Display)]
#[allow(dead_code)]
pub enum ClassFileVersion {
    Jdk1_1,
    Jdk1_2,
    Jdk1_3,
    Jdk1_4,
    Jdk1_5,
    #[default]
    Jdk6,
}

impl ClassFileVersion {
    pub fn from(major: u16, minor: u16) -> Result<ClassFileVersion> {
        match major {
            45u16 => Ok(ClassFileVersion::Jdk1_1),
            46u16 => Ok(ClassFileVersion::Jdk1_2),
            47u16 => Ok(ClassFileVersion::Jdk1_3),
            48u16 => Ok(ClassFileVersion::Jdk1_4),
            49u16 => Ok(ClassFileVersion::Jdk1_5),
            50u16 => Ok(ClassFileVersion::Jdk6),
            _ => Err(ClassReaderError::UnsupportedVersion(major, minor)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::class_file_version::ClassFileVersion;

    #[test]
    fn can_parse_known_versions() {
        assert_eq!(
            ClassFileVersion::Jdk6,
            ClassFileVersion::from(50, 0).unwrap()
        );
    }

    #[test]
    fn can_parse_future_versions() {
        assert_eq!(
            Err(crate::class_reader_error::ClassReaderError::UnsupportedVersion(62, 65535)),
            ClassFileVersion::from(62, 65535),
        );
    }
}
