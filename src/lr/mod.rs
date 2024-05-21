use std::fmt::Debug;

use crate::{ItemSetId, RuleId, Symbol};

mod transition;
mod action;
mod graph;
mod table;

use graph::*;
use transition::*;
pub use action::*;
pub use table::*;

#[derive(Debug)]
pub enum LrParserError<'sid, 'sym> {
    MissingRule(RuleId),
    MissingSet(ItemSetId),
    UnsupportedLrRank,
    ShiftReduceConflict {
        state: ItemSetId,
        symbol: &'sym Symbol<'sid>,
        conflict: [Action; 2]
    }
}

impl std::fmt::Display for LrParserError<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LrParserError::MissingRule(id) => write!(f, "Missing rule #{}", id),
            LrParserError::MissingSet(id) => write!(f, "Missing set #{}", id),
            LrParserError::ShiftReduceConflict { state, symbol, conflict } => write!(f, "Shift/reduce conflict for symbol {}, step #{} ({:?})", symbol.id, state, conflict),
            LrParserError::UnsupportedLrRank => write!(f, "Cannot build LR table for K > 1."),
        }
    }
}

pub type LrResult<'sid, 'sym, T> = Result<T, LrParserError<'sid, 'sym>>;

#[cfg(test)]
mod tests {
    use crate::fixtures::{fixture_lr0_grammar, fixture_lr1_grammar};

    use super::Table;

    #[test]
    pub fn test_lr0_grammar_table_building() {
        let g = fixture_lr0_grammar().expect("cannot build LR(0) grammar.");
        let table = Table::build::<0>(&g).expect("cannot build table");
        println!("{}", table);
    }

    #[test]
    pub fn test_lr1_grammar_table_building() {
        let g = fixture_lr1_grammar().expect("cannot build LR(1) grammar.");
        let table = Table::build::<1>(&g).expect("cannot build table");
        println!("{}", table);
    }
}