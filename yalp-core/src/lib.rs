pub mod ast;
pub mod dfa;
pub mod error;
pub mod grammar;
pub mod item;
pub mod lexer;
pub mod lr;
pub mod parser;
pub mod rule;
pub mod symbol;
pub mod token;

pub use grammar::ConstGrammar;
pub use lexer::*;
pub use rule::*;
pub use symbol::*;

pub(crate) use item::*;

pub use lr::{LrParser, LrTable};

pub use lexer::Span;

pub mod traits {
    pub use crate::lexer::traits::Lexer;
    pub use crate::parser::traits::{Ast, Parser};
    pub use crate::symbol::traits::SymbolSlice;
    pub use crate::token::traits::Token;
}

mod array;

pub use error::{ErrorKind, NoCustomError, YalpError};
pub type YalpResult<T, E> = Result<T, YalpError<E>>;

#[cfg(test)]
pub mod fixtures {
    use crate::{ConstGrammar, RuleDef, Symbol, EOS, START};

    pub const FIXTURE_LR1_GRAMMAR: ConstGrammar<'static, 9, 6> = ConstGrammar::new(
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

    pub const FIXTURE_LR0_GRAMMAR: ConstGrammar<'static, 9, 6> = ConstGrammar::new(
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
