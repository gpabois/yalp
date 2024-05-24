extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;

pub(crate) mod grammar;
pub(crate) mod symbol;
pub(crate) mod rule;

pub(crate) mod lexer;

pub(crate) use symbol::{SymbolIdentSet, parse_symbol_ident_set};
pub(crate) use grammar::parse_grammar;
pub(crate) use lexer::{Lexer, Token};
use yalp::{parser::ParserError, LexerError, LrParserError};

#[derive(Debug)]
pub(crate) enum Error {
    ParserError(LrParserError<'static, 'static>),
    LexerError(LexerError),
    WrongSymbol {
        expecting: String,
        got: String
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ParserError(err) => err.fmt(f),
            Error::LexerError(err) => err.fmt(f),
            Error::WrongSymbol { expecting, got } => write!(f, "wrong symbol: {expecting}, {got}"),
        }
    }
}

impl From<ParserError<LrParserError<'static, 'static>, Self>> for Error {
    fn from(value: ParserError<LrParserError<'static, 'static>, Self>) -> Self {
        match value {
            ParserError::Lexer(lexer) => Error::LexerError(lexer),
            ParserError::Parser(parser) => Error::ParserError(parser),
            ParserError::Custom(err) => err,
        }
    }
}

impl From<LrParserError<'static, 'static>> for Error {
    fn from(value: LrParserError<'static, 'static>) -> Self {
        Self::ParserError(value)
    }
}

impl Error {
    pub fn wrong_symbol(expecting: &str, got: &str) -> Self {
        Self::WrongSymbol {
            expecting: expecting.to_string(),
            got: got.to_string()
        }
    }
}

/// Declares a new grammar
///
/// # Example
/// ```
/// grammar! {
///     terminals: [<term>, "+", 0, 1],
///     non_terminals: [],
///     rules: {
///         <start> => E <eos>;
///         E => E "+" B;
///         E => B;
///         B => 0;
///         B => 1; 
///     }
/// }
/// ```
#[proc_macro]
pub fn grammar(stream: TokenStream) -> TokenStream {
    process_grammar_macro(stream.into()).into()
}

pub(crate) fn process_grammar_macro(stream: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let grammar_input = parse_grammar(stream).unwrap();
    quote!{}.into()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use proc_macro2::TokenStream;
    
    use super::process_grammar_macro;

    #[test]
    pub fn test_grammar_macro() {
        process_grammar_macro(TokenStream::from_str("
            terminals: [],
            non_terminals: [],
            rules: {}
        ").expect("cannot parse macro"));
    }
}