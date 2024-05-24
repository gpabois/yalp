pub mod traits {
    use crate::{lexer::LexerError, traits::Lexer};

    pub trait Ast {
        fn symbol_id(&self) -> &str;
    }
    pub trait Parser {
        type Ast: crate::traits::Ast;
        type Error: From<LexerError>;

        fn parse<L: Lexer>(&self, lexer: &mut L) -> Result<Self::Ast, Self::Error>
        where
            Self::Ast: From<L::Token>;
    }
}
