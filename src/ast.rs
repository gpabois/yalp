use crate::{parser::traits::Ast, token::Token, AstIter, Rule};

#[derive(Debug)]
pub struct AstNode<'kind> {
    kind: &'kind str,
    value: String,
    children: Vec<AstNode<'kind>>
}

impl<'kind> AstNode<'kind> {
    pub fn new<I: Iterator<Item=Self>>(kind: &'kind str, children: I) -> Self {
        Self {
            kind,
            value: String::default(),
            children: children.collect()
        }
    }

    pub fn reduce<'a, 'c, 'd>(rule: &'a Rule<'kind, 'c>, children: AstIter<'d, AstNode<'kind>>) -> AstNode<'kind> {
        AstNode::new(rule.lhs.id, children)
    }
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
            children: vec![]
        }
    }
}
