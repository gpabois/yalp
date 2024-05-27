pub mod traits {
    use crate::{traits::Lexer, YalpResult};

    pub trait Ast {
        fn symbol_id(&self) -> &str;
    }
    pub trait Parser<Error: Clone> {
        type Ast: crate::traits::Ast;

        fn parse<L: Lexer<Error>>(&self, lexer: &mut L) -> YalpResult<Self::Ast, Error>
        where
            Self::Ast: From<L::Token>;
    }
}
