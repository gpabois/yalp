pub mod grammar;
//pub mod lr;
pub mod rule;
pub mod symbol;
pub mod item;
pub mod lookahead;

mod array;

pub use item::*;
pub use lookahead::*;
pub use grammar::{Grammar, GrammarResult, GrammarError};
pub use rule::*;
pub use symbol::*;

#[cfg(test)]
pub mod fixtures {
    use crate::{Grammar, GrammarResult};

    pub fn fixture_lr1_grammar() -> GrammarResult<'static, Grammar<'static>> {
        let mut grammar = Grammar::default();

        grammar
            .add_terminal_symbol("(")?
            .add_terminal_symbol(")")?
            .add_terminal_symbol("n")?
            .add_terminal_symbol("+")?
            .add_non_terminal_symbol("E")?
            .add_non_terminal_symbol("T")?;

        grammar
            .add_rule("<start>", ["E", "<eos>"])?
            .add_rule("E", ["T"])?
            .add_rule("E", ["(", "E", ")"])?
            .add_rule("T", ["n"])?
            .add_rule("T", ["+", "T"])?
            .add_rule("T", ["T", "+", "n"])?;

        Ok(grammar)
    }
    pub fn fixture_lr0_grammar() -> GrammarResult<'static, Grammar<'static>> {
        let mut grammar = Grammar::default();

        grammar
            .add_terminal_symbol("0")?
            .add_terminal_symbol("1")?
            .add_terminal_symbol("+")?
            .add_terminal_symbol("*")?
            .add_non_terminal_symbol("E")?
            .add_non_terminal_symbol("B")?;

        grammar
            .add_rule("<start>", ["E", "<eos>"])?
            .add_rule("E", ["E", "*", "B"])?
            .add_rule("E", ["E", "+", "B"])?
            .add_rule("E", ["B"])?
            .add_rule("B", ["0"])?
            .add_rule("B", ["1"])?;

        Ok(grammar)
    }
}
