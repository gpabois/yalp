mod leaf;
mod sequence;
mod either;
mod group;
mod quantified;

use std::collections::HashMap;

pub use leaf::Leaf;
pub use sequence::Sequence;
pub use either::Either;
pub use quantified::Quantified;

use crate::{charset::CharSet, dfa};

/// A set of regular expressions.
pub struct RegexSet(HashMap<String, Regex>);

/// A regular expression
pub struct Regex(Expr);

#[derive(Debug, Clone)]
/// An action performed by the Regex's automaton when transitioning to another state.
pub enum Action {
    /// Consume the character
    Consume,
    
    /// Push a new group
    PushGroup {
        id: Option<String>
    },
    
    /// Pop the current group
    PopGroup,

    /// Match the sequence
    Match {
        regex_id: Option<String>,
        groups: HashMap<String, String>
    }
}

/// A regex sub-expression.
pub enum Expr {
    Sequence(Sequence),
    Either(Either),
    Quantified(Quantified),
    Leaf(Leaf),
}

impl dfa::IntoGraph<CharSet, Action>  for Expr {
    fn into_graph(self) -> dfa::Graph<CharSet, Action> {
        match self {
            Expr::Sequence(expr) => expr.into_graph(),
            Expr::Either(expr) => expr.into_graph(),
            Expr::Quantified(_) => todo!(),
            Expr::Leaf(expr) => expr.into_graph(),
        }
    }
}


