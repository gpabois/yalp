use crate::{parser, span::Span, token::Token};

pub struct AstNodeChildren<'stream>(Vec<AstNode<'stream>>);

impl<'stream> FromIterator<AstNode<'stream>> for AstNodeChildren<'stream> {
    fn from_iter<T: IntoIterator<Item = AstNode<'stream>>>(iter: T) -> Self {
        Self(Vec::from_iter(iter))
    }
}

impl AstNodeChildren<'_> {
    pub fn span(&self) -> Span {
        Span::from_iter(self.iter().map(|ast| ast.span).copied())
    }
}

#[derive(Debug)]
pub struct AstNode<'stream> {
    pub kind: String,
    pub value: Option<&'stream str>,
    pub children: AstNodeChildren<'stream>,
    pub span: Span,
}

impl<'stream> AstNode<'stream> {
    pub fn new<I: Iterator<Item = Self>>(kind: &str, children: I) -> Self {
        let children = AstNodeChildren::from_iter(children);

        Self {
            kind: kind.to_owned(),
            value: None,
            span: children.span(),
            children,
        }
    }
}

impl<'stream> parser::Ast for AstNode<'stream> {
    fn symbol_id(&self) -> &str {
        &self.kind
    }

    fn reduce(lhs: &str, rhs: impl Iterator<Item = Self>) -> Self {
        Self::new(lhs, rhs)
    }
}

impl<'stream> From<Token<'stream>> for AstNode<'stream> {
    fn from(token: Token<'stream>) -> Self {
        Self {
            kind: token.kind,
            value: Some(token.value),
            children: vec![],
            span: token.span,
        }
    }
}
