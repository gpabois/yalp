pub mod ast;
pub mod grammar;
pub mod item;
pub mod lexer;
pub mod lr;
pub mod parser;
pub mod rule;
pub mod symbol;
pub mod token;

pub use grammar::{Grammar, GrammarError, GrammarResult};
pub use item::*;
pub use lexer::*;
pub use rule::*;
pub use symbol::*;

pub use lr::{LrParser, LrParserError, LrTable};

pub use lexer::Span;

pub mod traits {
    pub use crate::lexer::traits::Lexer;
    pub use crate::parser::traits::{Ast, Parser};
    pub use crate::symbol::traits::SymbolSlice;
    pub use crate::token::traits::Token;
}

mod array;

#[derive(Debug)]
pub enum YalpError<Custom> {
    LexerError(LexerError),
    LrParserError(LrParserError),
    WrongSymbol { expecting: String, got: String },
    Custom(Custom),
}

impl<Custom> YalpError<Custom> {
    pub fn custom(value: Custom) -> Self {
        Self::Custom(value)
    }

    pub fn wrong_symbol(expecting: &str, got: &str) -> Self {
        Self::WrongSymbol {
            expecting: expecting.to_owned(),
            got: got.to_owned(),
        }
    }
}

impl<Custom> From<LrParserError> for YalpError<Custom> {
    fn from(value: LrParserError) -> Self {
        Self::LrParserError(value)
    }
}

impl<Custom> From<LexerError> for YalpError<Custom> {
    fn from(value: LexerError) -> Self {
        Self::LexerError(value)
    }
}

#[cfg(test)]
pub mod fixtures {
    use crate::{Grammar, RuleDef, Symbol, EOS, START};

    pub const FIXTURE_LR1_GRAMMAR: Grammar<'static, 9, 6> = Grammar::new(
        [
            Symbol::start(),
            Symbol::eos(),
            Symbol::epsilon(),
            Symbol::term("("),
            Symbol::term(")"),
            Symbol::term("n"),
            Symbol::term("+"),
            Symbol::nterm("E"),
            Symbol::nterm("T"),
        ],
        [
            RuleDef::new(START, &["E", EOS]),
            RuleDef::new("E", &["(", "E", ")"]),
            RuleDef::new("E", &["T"]),
            RuleDef::new("T", &["n"]),
            RuleDef::new("T", &["+", "T"]),
            RuleDef::new("T", &["T", "+", "n"]),
        ],
    );

    pub const FIXTURE_LR0_GRAMMAR: Grammar<'static, 9, 6> = Grammar::new(
        [
            Symbol::start(),
            Symbol::eos(),
            Symbol::epsilon(),
            Symbol::term("0"),
            Symbol::term("1"),
            Symbol::term("+"),
            Symbol::term("*"),
            Symbol::nterm("E"),
            Symbol::nterm("B"),
        ],
        [
            RuleDef::new(START, &["E", EOS]),
            RuleDef::new("E", &["E", "*", "B"]),
            RuleDef::new("E", &["E", "+", "B"]),
            RuleDef::new("E", &["B"]),
            RuleDef::new("B", &["0"]),
            RuleDef::new("B", &["1"]),
        ],
    );

    #[test]
    fn test_grammars() {
        println!("{:#?}", FIXTURE_LR1_GRAMMAR);
        println!("{:#?}", FIXTURE_LR0_GRAMMAR);
    }
}
