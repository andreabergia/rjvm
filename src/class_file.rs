use std::fmt;

use crate::constant_pool::ConstantPool;

bitflags! {
    pub struct ClassAccessFlags: u16 {
        const PUBLIC = 0x0001;
        const FINAL = 0x0010;
        const SUPER = 0x0020;
        const INTERFACE = 0x0200;
        const ABSTRACT = 0x0400;
        const SYNTHETIC = 0x1000;
        const ANNOTATION = 0x2000;
        const ENUM = 0x4000;
    }
}

impl Default for ClassAccessFlags {
    fn default() -> ClassAccessFlags {
        ClassAccessFlags::empty()
    }
}

/// Represents the content of a .class file.
#[derive(Debug, Default)]
pub struct ClassFile {
    pub major_version: u16,
    pub minor_version: u16,
    pub constants: ConstantPool,
    pub flags: ClassAccessFlags,
    pub name: String,
    pub superclass: String,
}

impl fmt::Display for ClassFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "Class {} (extends {}), version: {}.{}",
            self.name, self.superclass, self.major_version, self.minor_version
        )?;
        write!(f, "{}", self.constants)?;
        writeln!(f, "{:?}", self.flags)
    }
}

#[allow(dead_code)]
pub const JAVA1_CLASSFILE: u16 = 45;
#[allow(dead_code)]
pub const JAVA2_CLASSFILE: u16 = 46;
#[allow(dead_code)]
pub const JAVA3_CLASSFILE: u16 = 47;
#[allow(dead_code)]
pub const JAVA4_CLASSFILE: u16 = 48;
#[allow(dead_code)]
pub const JAVA5_CLASSFILE: u16 = 49;
#[allow(dead_code)]
pub const JAVA6_CLASSFILE: u16 = 50;
#[allow(dead_code)]
pub const JAVA7_CLASSFILE: u16 = 51;
#[allow(dead_code)]
pub const JAVA8_CLASSFILE: u16 = 52;
#[allow(dead_code)]
pub const JAVA9_CLASSFILE: u16 = 53;
