use std::{collections::HashSet, hash::Hash};

use itertools::Itertools;

use crate::{Rule, RuleSet, Symbol};

impl<'sid, 'sym> Rule<'sid, 'sym> {
    pub fn at<'rule>(&'rule self, position: usize) -> Option<Item<'sid, 'sym, 'rule>> {
        Item::new(self, position)
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
/// A rule item.
///
/// # Example
/// A -> w • eof
pub struct Item<'sid, 'sym, 'rule> {
    pub rule: &'rule Rule<'sid, 'sym>,
    pub position: usize,
}

impl Hash for Item<'_, '_, '_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.rule.hash(state);
        self.position.hash(state);
    }
}

impl<'sid, 'sym, 'rule> Item<'sid, 'sym, 'rule> {
    fn new(rule: &'rule Rule<'sid, 'sym>, position: usize) -> Option<Self> {
        if rule.rhs.len() >= position {
            Some(Self { rule, position })
        } else {
            None
        }
    }

    /// Check if we reached the end of a rule.
    ///
    /// # Example
    /// A -> w • eof
    pub fn is_terminating(&self) -> bool {
        self.position >= self.rule.rhs.len()
    }

    pub fn is_reaching_eos(&self) -> bool {
        self.symbol().unwrap().eos
    }

    /// Returns the current symbol.
    /// If A -> w • eof, then returns None.
    pub fn symbol(&self) -> Option<&'sym Symbol<'sid>> {
        self.rule.rhs.get(self.position).copied()
    }

    /// Returns the next rule's item.
    ///
    /// # Example
    /// (A -> • w eof).next() -> (A -> w • eof)
    pub fn next(&self) -> Option<Self> {
        Self::new(self.rule, self.position + 1)
    }
}

#[derive(Debug, Default)]
pub struct ItemSet<'sid, 'sym, 'rule> {
    pub id: usize,
    kernel: HashSet<Item<'sid, 'sym, 'rule>>,
    items: Vec<Item<'sid, 'sym, 'rule>>,
}

impl<'sid, 'sym, 'rule> PartialEq for ItemSet<'sid, 'sym, 'rule> {
    fn eq(&self, other: &Self) -> bool {
        self.kernel.eq(&other.kernel)
    }
}

impl<'sid, 'sym, 'rule> From<&'rule RuleSet<'sid, 'sym>> for ItemSet<'sid, 'sym, 'rule> {
    fn from(value: &'rule RuleSet<'sid, 'sym>) -> Self {
        value
            .iter_rules()
            .next()
            .and_then(|rule| rule.at(0))
            .into_iter()
            .collect()
    }
}

impl<'sid, 'sym, 'rule> FromIterator<Item<'sid, 'sym, 'rule>> for ItemSet<'sid, 'sym, 'rule> {
    /// Collect as the kernel's set.
    fn from_iter<T: IntoIterator<Item = Item<'sid, 'sym, 'rule>>>(iter: T) -> Self {
        Self {
            id: 0,
            kernel: iter.into_iter().collect(),
            items: vec![],
        }
    }
}

impl<'sid, 'sym, 'rule> ItemSet<'sid, 'sym, 'rule> {
    fn new<I1, I2>(kernel: I1, items: I2) -> Self
    where
        I1: IntoIterator<Item = Item<'sid, 'sym, 'rule>>,
        I2: IntoIterator<Item = Item<'sid, 'sym, 'rule>>,
    {
        Self {
            id: 0,
            kernel: kernel.into_iter().collect(),
            items: items.into_iter().collect(),
        }
    }

    /// Returns true if one of the item is terminating.
    pub fn has_terminating_item(&self) -> bool {
        self.iter().any(|item| item.is_terminating())
    }

    pub fn has_item_reaching_eos(&self) -> bool {
        self.iter().any(|item| item.is_reaching_eos())
    }

    pub fn get_terminating_rule(&self) -> usize {
        self.iter()
            .find(|item| item.is_terminating())
            .map(|item| item.rule.id)
            .unwrap()
    }

    pub fn next(&self) -> Self {
        self.iter().flat_map(Item::next).collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Item<'sid, 'sym, 'rule>> {
        return self.kernel.iter().chain(self.items.iter());
    }

    pub fn push(&mut self, item: Item<'sid, 'sym, 'rule>) {
        if !self.contains(&item) {
            self.items.push(item)
        }
    }

    pub fn contains(&self, item: &Item<'sid, 'sym, 'rule>) -> bool {
        self.kernel.contains(item) || self.items.contains(item)
    }

    pub fn pop(&mut self) -> Option<Item<'sid, 'sym, 'rule>> {
        self.items.pop()
    }

    pub fn append<I>(&mut self, items: I)
    where
        I: Iterator<Item = Item<'sid, 'sym, 'rule>>,
    {
        for item in items {
            self.push(item)
        }
    }

    /// Iterable over all reachable sets from the current set.
    ///
    /// The transition returns the symbol, and the kernel.
    pub fn reachable_sets(&self) -> Vec<(&'sym Symbol<'sid>, ItemSet<'sid, 'sym, 'rule>)> {
        self.iter()
            .group_by(|item| item.rule.lhs)
            .into_iter()
            .map(|(sym, items)| (sym, items.flat_map(|item| item.next()).collect()))
            .collect()
    }

    /// Close the item set
    ///
    /// It will fetch all rules until the next symbol is a terminal one, or we reach the end of a rule.
    pub fn close(&mut self, rules: &'rule RuleSet<'sid, 'sym>) {
        let mut stack: Vec<_> = self.kernel.clone().into_iter().collect();

        while let Some(item) = stack.pop() {
            if item.symbol().map(|sym| !sym.terminal).unwrap_or(false) {
                let sym = item.symbol().unwrap();
                for item in rules
                    .iter_symbol_related_rules(sym)
                    .flat_map(|rule| rule.at(0))
                {
                    if !self.contains(&item) {
                        stack.push(item.clone());
                        self.push(item);
                    }
                }
            }
        }
    }
}
