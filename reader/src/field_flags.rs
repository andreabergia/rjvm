bitflags! {
    /// Possible flags of a class field
    pub struct FieldFlags: u16 {
        const PUBLIC = 0x0001;
        const PRIVATE = 0x0002;
        const PROTECTED = 0x004;
        const STATIC = 0x0008;
        const FINAL = 0x0010;
        const VOLATILE = 0x0040;
        const TRANSIENT = 0x0080;
        const SYNTHETIC = 0x1000;
        const ENUM = 0x4000;
    }
}

impl Default for FieldFlags {
    fn default() -> FieldFlags {
        FieldFlags::empty()
    }
}
