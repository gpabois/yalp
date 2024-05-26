use std::convert::Infallible;

use crate::{parser::traits::Ast, token::Token, AstIter, Rule, YalpError};

#[derive(Debug)]
pub struct AstNode<'kind> {
    pub kind: &'kind str,
    pub value: String,
    pub children: Vec<AstNode<'kind>>,
}

impl<'kind> AstNode<'kind> {
    pub fn new<I: Iterator<Item = Self>>(kind: &'kind str, children: I) -> Self {
        Self {
            kind,
            value: String::default(),
            children: children.collect(),
        }
    }
}

pub fn ast_reduce<'a, 'b, 'c>(
    rule: &'a Rule<'b>,
    children: AstIter<'c, AstNode<'b>>,
) -> Result<AstNode<'b>, YalpError<Infallible>> {
    Ok(AstNode::new(rule.lhs.id, children))
}

impl<'kind> Ast for AstNode<'kind> {
    fn symbol_id(&self) -> &str {
        self.kind
    }
}

impl<'kind> From<Token<'kind>> for AstNode<'kind> {
    fn from(token: Token<'kind>) -> Self {
        Self {
            kind: token.kind,
            value: token.value,
            children: vec![],
        }
    }
}
