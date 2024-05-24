use proc_macro2::{Group, Ident, Literal, TokenStream, TokenTree};
use yalp::{lexer::LexerResult, traits::Token as _};
use crate::Error;

#[derive(Debug)]
pub(crate) struct Token(TokenTree);

impl yalp::token::traits::Token for Token {
    fn symbol_id(&self) -> &str {
        match &self.0 {
            TokenTree::Group(_) => "<group>",
            TokenTree::Ident(_) => "<ident>",
            TokenTree::Punct(punct) => match punct.to_string().as_str() {
                ":" => ":",
                "," => ",",
                ">" => ">",
                "<" => "<",
                "-" => "-",
                _ => "<illegal>"
            },
            TokenTree::Literal(_) => "<lit>",
        }
    }
}

impl TryFrom<Token> for Group {
    type Error = Error;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value.0 {
            TokenTree::Group(group) => Ok(group),
            _ => Err(Error::wrong_symbol("<group>", value.symbol_id()))
        }
    }
}

impl TryFrom<Token> for Ident {
    type Error = Error;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value.0 {
            TokenTree::Ident(ident) => Ok(ident),
            _ => Err(Error::wrong_symbol("<ident>", value.symbol_id()))
        }
    }
}

impl TryFrom<Token> for Literal {
    type Error = Error;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value.0 {
            TokenTree::Literal(lit) => Ok(lit),
            _ => Err(Error::wrong_symbol("<lit>", value.symbol_id()))
        }
    }
}

pub(crate) struct Lexer {
    current_span: yalp::Span,
    stream: proc_macro2::token_stream::IntoIter
}

impl Lexer {
    pub fn new(stream: TokenStream) -> Self {
        Self{
            stream: stream.into_iter(),
            current_span: yalp::Span::default()
        }
    }
}

impl Iterator for Lexer {
    type Item = LexerResult<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        let tt = self.stream.next()?;
        self.current_span = yalp::Span::new(0, 0);
        Some(Ok(Token(tt)))
    }
}

impl yalp::traits::Lexer for Lexer {
    type Token = Token;
    
    fn span(&self) -> yalp::Span {
        self.current_span
    }
}