use proc_macro2::{Group, Ident, Literal, TokenStream, TokenTree};
use yalp_core::{traits::Token as _, YalpError, YalpResult};

use crate::Error;

#[derive(Debug, Clone)]
pub(crate) struct Token(TokenTree);

impl yalp_core::token::traits::Token for Token {
    fn symbol_id(&self) -> &str {
        match &self.0 {
            TokenTree::Group(_) => "<group>",
            TokenTree::Ident(_) => "<ident>",
            TokenTree::Punct(punct) => match punct.to_string().as_str() {
                ":" => ":",
                "," => ",",
                ";" => ";",
                "=" => "=",
                ">" => ">",
                "<" => "<",
                "-" => "-",
                _ => "<illegal>",
            },
            TokenTree::Literal(_) => "<lit>",
        }
    }
}

impl TryFrom<Token> for Group {
    type Error = YalpError<Error>;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value.0 {
            TokenTree::Group(group) => Ok(group),
            _ => Err(yalp_core::ErrorKind::unexpected_symbol("<group>", [value.symbol_id()]).into()),
        }
    }
}

impl TryFrom<Token> for Ident {
    type Error = YalpError<Error>;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value.0 {
            TokenTree::Ident(ident) => Ok(ident),
            _ => Err(yalp_core::ErrorKind::unexpected_symbol("<ident>", [value.symbol_id()]).into()),
        }
    }
}

impl TryFrom<Token> for Literal {
    type Error = YalpError<Error>;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value.0 {
            TokenTree::Literal(lit) => Ok(lit),
            _ => Err(yalp_core::ErrorKind::unexpected_symbol("<lit>", [value.symbol_id()]).into()),
        }
    }
}

pub(crate) struct Lexer {
    current_span: yalp_core::Span,
    stream: proc_macro2::token_stream::IntoIter,
}

impl Lexer {
    pub fn new(stream: TokenStream) -> Self {
        Self {
            stream: stream.into_iter(),
            current_span: yalp_core::Span::default(),
        }
    }
}

impl Iterator for Lexer {
    type Item = YalpResult<Token, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let tt = self.stream.next()?;
        self.current_span = yalp_core::Span::new(0, 0);
        Some(Ok(Token(tt)))
    }
}

impl yalp_core::traits::Lexer<Error> for Lexer {
    type Token = Token;

    fn span(&self) -> yalp_core::Span {
        self.current_span
    }
}
