use std::fmt::Debug;

use crate::{RuleId, Symbol};

mod graph;
mod item;
mod table;

pub use table::*;

pub type ItemSetId = usize;

#[derive(Debug)]
pub enum LrParserError<'sid, 'sym> {
    MissingRule(RuleId),
    MissingSet(ItemSetId),
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
        }
    }
}

pub type LrResult<'sid, 'sym, T> = Result<T, LrParserError<'sid, 'sym>>;



#[cfg(test)]
mod tests {
    use super::{fixtures::fixture_grammar, Table};


    #[test]
    fn test_001_simple_table() {
        let g = fixture_grammar().expect("Cannot create grammar");
        let table = Table::<0>::build(&g).expect("Cannot build table");
    }
}