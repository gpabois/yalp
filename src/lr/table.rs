use super::item::ItemSet;
use crate::Symbol;

pub struct Transition<'sid, 'sym, 'rule, 'set> {
    pub from: &'set ItemSet<'sid, 'sym, 'rule>,
    pub edges: Vec<(&'sym Symbol<'sid>, &'set ItemSet<'sid, 'sym, 'rule>)>,
}

impl<'sid, 'sym, 'rule, 'set> Transition<'sid, 'sym, 'rule, 'set> {
    pub fn new<I>(from: &'set ItemSet<'sid, 'sym, 'rule>, edges: I) -> Self
    where
        I: Iterator<Item = (&'sym Symbol<'sid>, &'set ItemSet<'sid, 'sym, 'rule>)>,
    {
        Self {
            from,
            edges: edges.collect(),
        }
    }
}
