use crate::{charset::CharSet, dfa::{Graph, IntoGraph, Node}};

use super::Action;

#[derive(Clone)]
pub struct Leaf(CharSet);

impl From<CharSet> for Leaf {
    fn from(value: CharSet) -> Self {
        Self(value)
    }
}

impl IntoGraph<CharSet, Action> for Leaf {
    fn into_graph(self) -> crate::dfa::Graph<CharSet, Action> {
        let mut g = Graph::default();
        let n = g.add();
        g.on(Node::Start, n, self.0, [Action::Consume]);
        g.on(n, Node::End, CharSet::All, []);

        g
    }
}