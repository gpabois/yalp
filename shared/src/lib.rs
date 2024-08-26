use std::{borrow::Cow, ops::Deref};

#[derive(Debug, Clone)]
pub struct Symbol<'a>(Cow<'a, str>);

impl Deref for Symbol<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type StaticSymbol = Symbol<'static>;

impl From<String> for Symbol<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<&'a str> for Symbol<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

pub enum SyntaxSymbol<'a> {
    Terminal(Symbol<'a>),
    NonTerminal(Symbol<'a>),
    EOS,
}
