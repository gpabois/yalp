use super::{item::ItemSet, table::Transition, LrParserError, LrResult};
use crate::{RuleSet, Symbol};

pub struct Graph<'sid, 'sym, 'rule> {
    rules: &'rule RuleSet<'sid, 'sym>,
    sets: Vec<ItemSet<'sid, 'sym, 'rule>>,
    transitions: Vec<(usize, &'sym Symbol<'sid>, usize)>,
}

impl<'sid, 'sym, 'rule> Graph<'sid, 'sym, 'rule> {
    pub fn new(rules: &'rule RuleSet<'sid, 'sym>) -> Self {
        Self {
            rules,
            sets: vec![ItemSet::from_iter([rules.get(0).at(0).unwrap()])],
            transitions: vec![],
        }
    }
    /// Returns true if a set has the same kernel.
    fn contains(&self, set: &ItemSet<'sid, 'sym, 'rule>) -> bool {
        self.sets.iter().any(|s| s == set)
    }

    fn get_mut(&mut self, id: usize) -> Option<&mut ItemSet<'sid, 'sym, 'rule>> {
        self.sets.get_mut(id)
    }

    fn get(&self, id: usize) -> Option<&ItemSet<'sid, 'sym, 'rule>> {
        self.sets.get(id)
    }

    fn get_id(&self, kernel: &ItemSet<'sid, 'sym, 'rule>) -> Option<usize> {
        self.sets
            .iter()
            .find(|set| *set == kernel)
            .map(|set| set.id)
    }

    /// Push a new set in the graph, if it does not yet exist.
    fn push(&mut self, mut set: ItemSet<'sid, 'sym, 'rule>) -> usize {
        if !self.contains(&set) {
            let id = self.sets.len();
            set.id = id;
            self.sets.push(set);
            return id;
        }

        self.get_id(&set).unwrap()
    }

    pub fn build(&mut self) -> LrResult<'sid, 'sym, ()> {
        let mut stack = vec![0];
        let rules = self.rules;

        while let Some(set_id) = stack.pop() {
            self
                .get_mut(set_id)
                .ok_or(LrParserError::MissingSet(set_id))?
                .close(rules);

            for (symbol, kernel) in self.get(set_id).ok_or(LrParserError::MissingSet(set_id))?.reachable_sets() {
                let to_id = if !self.contains(&kernel) {
                    let id = self.push(kernel);
                    stack.push(id);
                    id
                } else {
                    self.get_id(&kernel).unwrap()
                };

                self.transitions.push((set_id, symbol, to_id));
            }
        }

        Ok(())
    }

    /// Iterate over all transition's table rows.
    pub fn iter_transitions<'set>(
        &'set self,
    ) -> impl Iterator<Item = Transition<'sid, 'sym, 'rule, 'set>> {
        self.sets.iter().map(|set| {
            Transition::new(
                set,
                self.transitions
                    .iter()
                    .filter(|t| t.0 == set.id)
                    .map(|t| (t.1, self.get(t.2).unwrap())),
            )
        })
    }
}
