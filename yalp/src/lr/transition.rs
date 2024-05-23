use crate::{ItemSet, Symbol};

use super::graph::Graph;

pub struct Transition<'sid, 'sym, 'rule, 'set, const K: usize> {
    pub(super) from: &'set ItemSet<'sid, 'sym, 'rule, K>,
    pub(super) edges: Vec<(&'sym Symbol<'sid>, &'set ItemSet<'sid, 'sym, 'rule, K>)>
}

impl<'sid, 'sym, 'rule, const K: usize> Graph<'sid, 'sym, 'rule, K> {
    pub fn iter_transitions(&self)  -> impl Iterator<Item=Transition<'sid, 'sym, 'rule, '_, K>>  {
        self.sets.iter().map(|set| Transition {
            from: set,
            edges: self
                .transitions
                .iter()
                .filter(|(from, _, _)| set.id == *from)
                .map(|(_, sym, to)| (*sym, self.sets.get(*to).unwrap()))
                .collect()
        })
    }
    
}