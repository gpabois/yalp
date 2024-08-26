use itertools::Itertools as _;
use thiserror::Error;

use crate::{OwnedSymbol, Span};

#[derive(Debug, Clone, Copy)]
pub struct NoCustomError;

#[derive(Debug, Clone)]
pub struct ExpectedSymbols(Vec<String>);

impl std::fmt::Display for ExpectedSymbols {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.iter().join(", ").fmt(f)
    }
}

#[derive(Error, Debug, Clone)]
pub enum ErrorKind<C> {
    
    #[error("unknown rule {0}")]
    UnknownRule(usize),

    #[error("a symbol with the same identifier already exists {0}")]
    DuplicatedSymbolId(String),

    #[error("unknown symbol {0}")]
    UnknownSymbol(String),
    
    #[error("unexpected symbol {got}, expecting {expecting}")]
    UnexpectedSymbol {
        expecting: ExpectedSymbols,
        got: String
    },
    
    #[error("unexpected end of stream")]
    UnexpectedEndOfStream,
    
    #[error("a shift-reduce conflict has occurred for symbol {symbol} [{conflict:?}], state={state}")]
    ShiftReduceConflict{
        state: usize,
        symbol: OwnedSymbol,
        conflict: [crate::lr::Action; 2],
    },

    #[error("the algorithm is not supported")]
    UnsupportedAlgorithm,

    #[error("{0}")]
    Other(C)
}

impl<C> ErrorKind<C> {
    pub fn unexpected_symbol<I, S>(got: &str, expecting: I) -> Self
        where I: IntoIterator<Item=S>, 
            S: ToString {
        Self::UnexpectedSymbol { 
            expecting: ExpectedSymbols(expecting.into_iter().map(|s| s.to_string()).collect()), 
            got: got.to_string() 
        }
    }

    pub fn unknown_symbol(got: &str) -> Self {
        Self::UnknownSymbol(got.to_string())
    }
}

#[derive(Error, Debug, Clone)]
pub struct YalpError<C> {
    /// Kind of error
    kind: ErrorKind<C>,
    /// Location of the error in a stream.
    pub(crate) span: Option<Span>
}

impl<C> YalpError<C> {
    pub fn new(kind: impl Into<ErrorKind<C>>, span: Option<Span>) -> Self {
        Self {
            kind: kind.into(),
            span
        }
    }
}

impl<C> From<ErrorKind<C>> for YalpError<C> {
    fn from(kind: ErrorKind<C>) -> Self {
        Self {
            kind,
            span: None
        }
    }
}

impl<C> YalpError<C> {
    pub fn kind(&self) -> &ErrorKind<C> {
        &self.kind
    }

    pub fn span(&self) -> Option<Span> {
        self.span.clone()
    }
}