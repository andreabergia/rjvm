use std::{
    fmt,
    fmt::{Display, Formatter},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct ProgramCounter(pub u16);

impl Display for ProgramCounter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
