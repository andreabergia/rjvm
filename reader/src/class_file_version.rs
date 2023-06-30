use crate::class_reader_error::{ClassReaderError, Result};

/// Versions of the JVM class file format.
#[derive(Debug, PartialEq, Default, strum_macros::Display)]
#[allow(dead_code)]
pub enum ClassFileVersion {
    Jdk1_1,
    Jdk1_2,
    Jdk1_3,
    Jdk1_4,
    Jdk1_5,
    Jdk6,
    Jdk7,
    #[default]
    Jdk8,
    Jdk9,
    Jdk10,
    Jdk11,
    Jdk12,
    Jdk13,
    Jdk14,
    Jdk15,
    Jdk16,
    Jdk17,
    Jdk18,
    Jdk19,
    Jdk20,
    Jdk21,
    Jdk22,
}

impl ClassFileVersion {
    /// Creates a version from the major and minor versions specified in the class file
    pub fn from(major: u16, minor: u16) -> Result<ClassFileVersion> {
        match major {
            45 => Ok(ClassFileVersion::Jdk1_1),
            46 => Ok(ClassFileVersion::Jdk1_2),
            47 => Ok(ClassFileVersion::Jdk1_3),
            48 => Ok(ClassFileVersion::Jdk1_4),
            49 => Ok(ClassFileVersion::Jdk1_5),
            50 => Ok(ClassFileVersion::Jdk6),
            51 => Ok(ClassFileVersion::Jdk7),
            52 => Ok(ClassFileVersion::Jdk8),
            53 => Ok(ClassFileVersion::Jdk9),
            54 => Ok(ClassFileVersion::Jdk10),
            55 => Ok(ClassFileVersion::Jdk11),
            56 => Ok(ClassFileVersion::Jdk12),
            57 => Ok(ClassFileVersion::Jdk13),
            58 => Ok(ClassFileVersion::Jdk14),
            59 => Ok(ClassFileVersion::Jdk15),
            60 => Ok(ClassFileVersion::Jdk16),
            61 => Ok(ClassFileVersion::Jdk17),
            62 => Ok(ClassFileVersion::Jdk18),
            63 => Ok(ClassFileVersion::Jdk19),
            64 => Ok(ClassFileVersion::Jdk20),
            65 => Ok(ClassFileVersion::Jdk21),
            66 => Ok(ClassFileVersion::Jdk22),
            _ => Err(ClassReaderError::UnsupportedVersion(major, minor)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{class_file_version::ClassFileVersion, class_reader_error::ClassReaderError};

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
            Err(ClassReaderError::UnsupportedVersion(99, 65535)),
            ClassFileVersion::from(99, 65535),
        );
    }
}
