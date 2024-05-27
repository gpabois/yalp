use std::collections::VecDeque;

use crate::{ItemSet, ItemSetId, RuleSet, Symbol, YalpResult};

pub struct Graph<'sid, 'sym, 'rule, const K: usize> {
    rules: &'rule RuleSet<'sid, 'sym>,
    pub(super) sets: Vec<ItemSet<'sid, 'rule, K>>,
    pub(super) edges: Vec<(ItemSetId, Symbol<'sid>, ItemSetId)>,
}

impl<'sid, 'sym, 'rule, const K: usize> Graph<'sid, 'sym, 'rule, K> {
    pub fn new(rules: &'rule RuleSet<'sid, 'sym>) -> Self {
        Self {
            rules,
            sets: vec![rules.start_item_set()],
            edges: vec![],
        }
    }

    /// Returns true if a set has the same kernel.
    fn contains(&self, set: &ItemSet<'sid, 'rule, K>) -> bool {
        self.sets.iter().any(|s| s == set)
    }

    fn get_mut(&mut self, id: usize) -> Option<&mut ItemSet<'sid, 'rule, K>> {
        self.sets.get_mut(id)
    }

    fn get(&self, id: usize) -> Option<&ItemSet<'sid, 'rule, K>> {
        self.sets.get(id)
    }

    fn get_id(&self, kernel: &ItemSet<'sid, 'rule, K>) -> Option<usize> {
        self.sets
            .iter()
            .find(|set| *set == kernel)
            .map(|set| set.id)
    }

    /// Push a new set in the graph, if it does not yet exist.
    fn push(&mut self, mut set: ItemSet<'sid, 'rule, K>) -> usize {
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
