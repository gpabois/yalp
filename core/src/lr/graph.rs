use std::collections::VecDeque;

use crate::{
    syntax::{PrepSymbol, PrepSyntax},
    ItemSet, ItemSetId, YalpResult,
};

pub struct Graph<'syntax, 'gen, const K: usize> {
    pub(super) rules: &'gen PrepSyntax<'syntax>,
    pub(super) sets: Vec<ItemSet<'syntax, 'gen, K>>,
    pub(super) edges: Vec<(ItemSetId, PrepSymbol<'syntax>, ItemSetId)>,
}

impl<'syntax, 'gen, const K: usize> Graph<'syntax, 'gen, K> {
    pub fn new(rules: &'gen PrepSyntax<'syntax>) -> Self {
        Self {
            rules,
            sets: vec![rules.start_item_set()],
            edges: vec![],
        }
    }

    /// Returns true if a set has the same kernel.
    fn contains(&self, set: &ItemSet<'syntax, 'gen, K>) -> bool {
        self.sets.iter().any(|s| s == set)
    }

    fn get_mut(&mut self, id: usize) -> Option<&mut ItemSet<'syntax, 'gen, K>> {
        self.sets.get_mut(id)
    }

    fn get(&self, id: usize) -> Option<&ItemSet<'syntax, 'gen, K>> {
        self.sets.get(id)
    }

    fn get_id(&self, kernel: &ItemSet<'syntax, 'gen, K>) -> Option<usize> {
        self.sets
            .iter()
            .find(|set| *set == kernel)
            .map(|set| set.id)
    }

    /// Push a new set in the graph, if it does not yet exist.
    fn push(&mut self, mut set: ItemSet<'syntax, 'gen, K>) -> usize {
        if !self.contains(&set) {
            let id = self.sets.len();
            set.id = id;
            self.sets.push(set);
            return id;
        }

        self.get_id(&set).unwrap()
    }

    pub fn build<Error>(&mut self) -> YalpResult<(), Error> {
        let mut stack = VecDeque::from_iter([0]);
        let rules = self.rules;

        while let Some(set_id) = stack.pop_front() {
            self.get_mut(set_id)
                .unwrap_or_else(|| panic!("Missing state {set_id}"))
                .close(rules);

            for (symbol, kernel) in self
                .get(set_id)
                .unwrap_or_else(|| panic!("Missing state {set_id}"))
                .reachable_sets(rules)
            {
                let to_id = if !self.contains(&kernel) {
                    let id = self.push(kernel);
                    stack.push_back(id);
                    id
                } else {
                    self.get_id(&kernel).unwrap()
                };

                self.edges.push((set_id, symbol, to_id));
            }
        }

        Ok(())
    }
}
