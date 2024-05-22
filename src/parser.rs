use crate::lexer::{traits::Lexer, LexerError};

pub mod traits {
    pub trait Ast {
        fn symbol_id(&self) -> &str;
    }   
}

pub trait Parser {
    type Ast: traits::Ast;
    type Error: From<LexerError>;

    fn parse<L: Lexer>(&self, lexer: &mut L) -> Result<Self::Ast, Self::Error>
    where Self::Ast: From<L::Token>;
}