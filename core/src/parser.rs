use crate::{traits::Lexer, YalpResult};

pub trait Ast {
    fn symbol_id(&self) -> &str;
    fn reduce(lhs: &str, rhs: impl Iterator<Item = Self>) -> Self;
}

pub trait Parser<Error: Clone> {
    type Ast: Ast;

    fn parse<L: Lexer<Error>>(&self, lexer: &mut L) -> YalpResult<Self::Ast, Error>
    where
        Self::Ast: From<L::Token>;
}
