use crate::span::Span;

pub mod traits {
    pub trait Token: Clone {
        fn symbol_id(&self) -> &str;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<'stream> {
    pub kind: String,
    pub value: &'stream str,
    pub span: Span,
}

impl<'kind> traits::Token for Token<'kind> {
    fn symbol_id(&self) -> &str {
        &self.kind
    }
}

impl<'stream> Token<'stream> {
    pub fn new<S>(kind: S, value: &'stream str, span: Span) -> Self
    where
        S: ToString,
    {
        Self {
            kind: kind.to_string(),
            value,
            span,
        }
    }
}
