use itertools::Itertools;

use crate::{ItemSet, Symbol};

use super::graph::Graph;

pub struct Transition<'sid, 'sym, 'rule, 'set, const K: usize> {
    pub(super) from: &'set ItemSet<'sid, 'sym, 'rule, K>,
    pub(super) edges: Vec<(&'sym Symbol<'sid>, &'set ItemSet<'sid, 'sym, 'rule, K>)>
}

impl<'sid, 'sym, 'rule, const K: usize> Graph<'sid, 'sym, 'rule, K> {
    pub fn iter_transitions(&self)  -> Vec<Transition<'sid, 'sym, 'rule, '_, K>>  {
        self.transitions
        .iter()
        .group_by(|t| t.0)
        .into_iter()
        .map(|(from, edges)| {
            Transition {
                from: self.sets.get(from).unwrap(),
                edges: edges.into_iter().map(|(_, sym, to)| (*sym, self.sets.get(*to).unwrap())).collect()
            }
        })
        .collect()
    }
    
}