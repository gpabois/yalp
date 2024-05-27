use crate::{parser::traits::Ast, rule::traits::RuleReducer, token::Token, RuleRhs, YalpResult};

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


pub struct AstNodeReducer;

impl<'kind, Error> RuleReducer<'kind, Error> for AstNodeReducer {
    type Ast = AstNode<'kind>;

    fn reduce(&self, rule: &crate::Rule<'kind>, rhs: RuleRhs<Self::Ast>) -> YalpResult<Self::Ast, Error> {
        Ok(AstNode::<'kind> {
            kind: rule.lhs.id,
            value: String::default(),
            children: rhs.into_iter().collect()
        })
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
            children: vec![],
        }
    }
}
