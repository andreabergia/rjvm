use std::ops::Range;

use crate::program_counter::ProgramCounter;

#[derive(Debug, Default, PartialEq)]
pub struct ExceptionTable {
    entries: Vec<ExceptionTableEntry>,
}

impl ExceptionTable {
    pub fn new(entries: Vec<ExceptionTableEntry>) -> Self {
        Self { entries }
    }

    pub fn lookup(&self, pc: ProgramCounter) -> Vec<&ExceptionTableEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.range.contains(&pc))
            .collect()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExceptionTableEntry {
    pub range: Range<ProgramCounter>,
    pub handler_pc: ProgramCounter,
    pub catch_class: Option<String>,
}

#[cfg(test)]
mod tests {
    use crate::{
        exception_table::{ExceptionTable, ExceptionTableEntry},
        program_counter::ProgramCounter,
    };

    #[test]
    fn can_lookup_catch_handler() {
        let entry_1 = ExceptionTableEntry {
            range: ProgramCounter(0)..ProgramCounter(4),
            handler_pc: ProgramCounter(99),
            catch_class: None,
        };
        let entry_2 = ExceptionTableEntry {
            range: ProgramCounter(8)..ProgramCounter(14),
            handler_pc: ProgramCounter(88),
            catch_class: Some("java/lang/RuntimeException".to_string()),
        };
        let entry_3 = ExceptionTableEntry {
            range: ProgramCounter(13)..ProgramCounter(14),
            handler_pc: ProgramCounter(77),
            catch_class: Some("java/lang/ClassCastException".to_string()),
        };
        let table = ExceptionTable::new(vec![entry_1.clone(), entry_2.clone(), entry_3.clone()]);

        assert_eq!(vec![&entry_1], table.lookup(ProgramCounter(0)));
        assert_eq!(vec![&entry_1], table.lookup(ProgramCounter(1)));
        assert!(table.lookup(ProgramCounter(4)).is_empty());
        assert_eq!(vec![&entry_2], table.lookup(ProgramCounter(8)));
        assert_eq!(vec![&entry_2, &entry_3], table.lookup(ProgramCounter(13)));
        assert!(table.lookup(ProgramCounter(14)).is_empty());
    }
}
