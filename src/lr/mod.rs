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
pub mod fixtures {
    use crate::{Grammar, GrammarResult};

    pub fn fixture_grammar() -> GrammarResult<'static, Grammar<'static>> {
        let mut grammar = Grammar::default();

        grammar
            .add_terminal_symbol("0")?
            .add_terminal_symbol("1")?
            .add_terminal_symbol("+")?
            .add_terminal_symbol("*")?
            .add_non_terminal_symbol("E")?
            .add_non_terminal_symbol("B")?;

        grammar
            .add_rule("<root>", ["E", "<eos>"])?
            .add_rule("E", ["E", "*", "B"])?
            .add_rule("E", ["E", "+", "B"])?
            .add_rule("E", ["B"])?
            .add_rule("B", ["0"])?
            .add_rule("B", ["1"])?;

        Ok(grammar)
    }
}

#[cfg(test)]
mod tests {
    use super::{fixtures::fixture_grammar, Action, Row, Table};


    #[test]
    fn test_001_simple_table() {
        let g = fixture_grammar().expect("Cannot create grammar");
        let table = Table::build(&g).expect("Cannot build table");
    }
}