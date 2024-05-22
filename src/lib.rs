pub mod grammar;
pub mod item;
pub mod lr;
pub mod rule;
pub mod symbol;
pub mod token;
pub mod lexer;
pub mod parser;
pub mod ast;

pub use grammar::{Grammar, GrammarError, GrammarResult};
pub use item::*;
pub use rule::*;
pub use symbol::*;

mod array;

#[cfg(test)]
pub mod fixtures {
    use crate::{nterm, rule, term, Grammar, GrammarResult};

    /// A real-life grammar.
    pub fn css_selector_grammar() -> GrammarResult<'static, Grammar<'static, 0, 17>> {
        let mut grammar = Grammar::new(
            [
                nterm!("<selector-list>"),
                nterm!("<complex-selector-list>"),
                nterm!("<complex-selector>"),
                nterm!("<compound-selector>"),
                nterm!("<combinator>"),
                nterm!("<type-selector>"),
                nterm!("<subclass-selector>"),
                nterm!("<pseudo-element-selector>"),
                nterm!("<pseudo-class-selector>"),
                nterm!("<wq-name>"),
                nterm!("<ns-prefix>"),
                term!("*"),
                term!("<ident-token>"),
                term!(">"),
                term!("+"),
                term!("~"),
                term!("|")
            ],
            []
        );

        Ok(grammar)
    }

    pub fn fixture_lr1_grammar() -> GrammarResult<'static, Grammar<'static, 6, 6>> {
        let mut grammar = Grammar::new([
            term!("("),
            term!(")"),
            term!("n"),
            term!("+"),
            nterm!("E"),
            nterm!("T")
        ], [
            rule!("<start>" => ["E", "<eos>"]),
            rule!("E" => ["T"]),
            rule!("E" => ["(", "E", ")"]),
            rule!("T" => ["n"]),
            rule!("T" => ["+", "T"]),
            rule!("T" => ["T", "+", "n"])
        ]);
        
        Ok(grammar)
    }
    pub fn fixture_lr0_grammar() -> GrammarResult<'static, Grammar<'static>> {
        let mut grammar = Grammar::default();

        grammar
            .add_terminal_symbol("0")?
            .add_terminal_symbol("1")?
            .add_terminal_symbol("*")?
            .add_terminal_symbol("+")?
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
