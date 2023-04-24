use std::cmp::Ordering;

use crate::line_number::LineNumber;
use itertools::Itertools;

use crate::program_counter::ProgramCounter;

#[derive(Debug, PartialEq)]
pub struct LineNumberTable {
    entries: Vec<LineNumberTableEntry>,
}

impl LineNumberTable {
    pub fn new(entries: Vec<LineNumberTableEntry>) -> Self {
        Self {
            entries: entries.into_iter().sorted().collect(),
        }
    }

    pub fn lookup_pc(&self, pc: ProgramCounter) -> LineNumber {
        let best_matching_entry_index = match self
            .entries
            .binary_search_by(|e| e.program_counter.cmp(&pc))
        {
            Ok(index) => index,
            Err(index) => index - 1,
        };
        self.entries[best_matching_entry_index].line_number
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LineNumberTableEntry {
    pub program_counter: ProgramCounter,
    pub line_number: LineNumber,
}

impl PartialOrd for LineNumberTableEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.program_counter.partial_cmp(&other.program_counter)
    }
}

impl Ord for LineNumberTableEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.program_counter.cmp(&other.program_counter)
    }
}

impl LineNumberTableEntry {
    pub fn new(program_counter: ProgramCounter, line_number: LineNumber) -> Self {
        Self {
            program_counter,
            line_number,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        line_number::LineNumber,
        line_number_table::{LineNumberTable, LineNumberTableEntry},
        program_counter::ProgramCounter,
    };

    #[test]
    fn can_lookup_line_number() {
        let table = LineNumberTable::new(vec![
            LineNumberTableEntry::new(ProgramCounter(0), LineNumber(4)),
            LineNumberTableEntry::new(ProgramCounter(12), LineNumber(5)),
            LineNumberTableEntry::new(ProgramCounter(20), LineNumber(6)),
        ]);

        assert_eq!(LineNumber(4), table.lookup_pc(ProgramCounter(0)));
        assert_eq!(LineNumber(4), table.lookup_pc(ProgramCounter(11)));
        assert_eq!(LineNumber(5), table.lookup_pc(ProgramCounter(12)));
        assert_eq!(LineNumber(6), table.lookup_pc(ProgramCounter(20)));
        assert_eq!(LineNumber(6), table.lookup_pc(ProgramCounter(21)));
    }
}
