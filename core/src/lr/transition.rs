use crate::{item::ItemSet, syntax::PrepSymbol};

use super::graph::Graph;

pub struct Transition<'syntax, 'gen, 'graph, const K: usize> {
    pub(super) from: &'graph ItemSet<'syntax, 'gen, K>,
    pub(super) edges: Vec<(PrepSymbol<'syntax>, &'graph ItemSet<'syntax, 'gen, K>)>,
}

impl<'syntax, 'gen, const K: usize> Graph<'syntax, 'gen, K> {
    pub fn iter_transitions(&self) -> impl Iterator<Item = Transition<'syntax, 'gen, '_, K>> {
        self.sets.iter().map(|set| Transition {
            from: set,
            edges: self
                .edges
                .iter()
                .filter(|(from, _, _)| set.id == *from)
                .map(|(_, sym, to)| (*sym, self.sets.get(*to).unwrap()))
                .collect(),
        })
    }
}
