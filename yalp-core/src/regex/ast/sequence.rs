use crate::{charset::CharSet, dfa::{Graph, IntoGraph}};

use super::{Action, Expr};

/// A1..An
pub struct Sequence(Vec<Expr>);

impl IntoIterator for Sequence {
    type Item = Expr;
    type IntoIter = <Vec<Expr> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl IntoGraph<CharSet, Action> for Sequence {
    fn into_graph(self) -> Graph<CharSet, Action> {
        self.into_iter()
            .map(IntoGraph::into_graph)
            .reduce(|mut a, b| {
                a.append(b);
                a
            })
            .unwrap_or_default()
    }
}