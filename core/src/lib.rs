pub mod ast;
//pub mod dfa;:
pub mod error;
pub mod item;
pub mod lexer;
pub mod lr;
pub mod parser;
pub mod prelude;
pub mod span;
pub mod syntax;
pub mod token;
pub use lexer::*;

pub(crate) use item::*;

pub use lexer::Span;

mod array;

pub use error::{ErrorKind, NoCustomError, YalpError};

pub type YalpResult<T, E> = Result<T, YalpError<E>>;

macro_rules! rule {
    ($lhs:literal ::= $($rhs:literal)*) => {
        StaticRule::new(StaticSymbol::new($lhs),
            &[
                $(StaticSymbol::new($rhs),)*
            ]
        )
    };

}

#[cfg(test)]
pub mod fixtures {

    use crate::syntax::{StaticRule, StaticSymbol, StaticSyntax};

    pub const FIXTURE_LR1_GRAMMAR: StaticSyntax = StaticSyntax::new(&[
        rule!("START" ::= "E"),
        rule!("E" ::= "(" "E" ")"),
        rule!("E" ::= "T"),
        rule!("T" ::= "n"),
        rule!("T" ::= "+" "T"),
        rule!("T" ::= "+" "+" "n"),
    ]);

    pub const FIXTURE_LR0_GRAMMAR: StaticSyntax = StaticSyntax::new(&[
        rule!("START" ::= "E"),
        rule!("E" ::= "E" "*" "B"),
        rule!("E" ::= "E" "+" "B"),
        rule!("E" ::= "B"),
        rule!("B" ::= "0"),
        rule!("B" ::= "1"),
    ]);

    #[test]
    fn test_grammars() {
        println!("{:#?}", FIXTURE_LR1_GRAMMAR);
        println!("{:#?}", FIXTURE_LR0_GRAMMAR);
    }
}
