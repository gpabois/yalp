
use crate::{charset::CharSet, dfa::{Graph, IntoGraph}};

use super::{Action, Expr};


/// A1 | A2 | ... | An
pub struct Either(Vec<Expr>);

impl IntoIterator for Either {
    type Item = Expr;
    type IntoIter = <Vec<Expr> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl IntoGraph<CharSet, Action> for Either {
    fn into_graph(self) -> Graph<CharSet, Action> {
        self.into_iter()
            .map(IntoGraph::into_graph)
            .reduce(Graph::merge)
            .unwrap_or_default()
    }
}
