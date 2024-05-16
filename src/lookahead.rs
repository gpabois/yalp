use crate::{array::Array, Symbol};


#[derive(Debug, PartialEq, Eq)]
pub struct Lookahead<'sid, 'sym, const K: usize> {
    first: &'sym Symbol<'sid>,
    others: Array<K, &'sym Symbol<'sid>>
}

impl<'sid, 'sym, const K: usize> std::hash::Hash for Lookahead<'sid, 'sym, K> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.first.hash(state);
        self.others.hash(state);
    }
}
