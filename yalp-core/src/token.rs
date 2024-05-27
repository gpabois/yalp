use crate::lexer::Span;

pub mod traits {
    pub trait Token: Clone {
        fn symbol_id(&self) -> &str;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<'kind> {
    pub kind: &'kind str,
    pub value: String,
    pub location: Span,
}

impl<'kind> traits::Token for Token<'kind> {
    fn symbol_id(&self) -> &str {
        self.kind
    }
}

impl<'kind> Token<'kind> {
    pub fn new<S>(kind: &'kind str, value: S, location: Span) -> Self
    where
        S: ToString,
    {
        Self {
            kind,
            value: value.to_string(),
            location,
        }
    }
}

