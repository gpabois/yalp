use std::{collections::HashSet, hash::Hash};

use itertools::Itertools;

use crate::{array::Array, Rule, RuleSet, Symbol};

impl<'sid, 'sym> Rule<'sid, 'sym> {
    pub fn at<'rule, const K: usize>(
        &'rule self,
        position: usize,
    ) -> Option<Item<'sid, 'sym, 'rule, K>> {
        Item::new(self, position)
    }
}

impl<'sid, 'sym> RuleSet<'sid, 'sym> {
    /// Recursively fetch and append dot lookaheads
    fn rec_follow<'rule, const K: usize>(
        &'rule self,
        source: &Array<K, &'sym Symbol<'sid>>,
        item: ItemCore<'sid, 'sym, 'rule>,
    ) -> Vec<Array<K, &'sym Symbol<'sid>>> {
        let mut arrays: Vec<Array<K, &'sym Symbol<'sid>>> = vec![];

        // No more space...
        if source.is_full() {
            return vec![source.clone()];
        }

        for (csym, citem) in item.dot_lookahead(self) {
            let mut carr = source.clone();
            carr.push(csym);

            // Cannot go further
            if carr.is_full() {
                arrays.push(carr);
            }
            // Check the next item's follow set.
            else if let Some(nitem) = citem.next() {
                arrays.extend(self.rec_follow(&carr, nitem));
            }
            // No more item possible.
            else {
                arrays.push(carr);
            }
        }

        arrays
    }

    // Fetch the new k terminal symbols from deriving the given non-terminal symbol.
    pub fn first<'rule, const K: usize>(
        &'rule self,
        from: &'sym Symbol<'sid>,
    ) -> HashSet<Array<K, &'sym Symbol<'sid>>> {
        if K == 0 {
            return Default::default();
        }

        let mut set: ItemSet<'sid, 'sym, 'rule, 0> = self
            .iter_symbol_related_rules(from)
            .flat_map(|rule| rule.at::<0>(0))
            .collect();

        set.close(self);

        if K == 1 {
            return set
                .iter_lookaheads()
                .map(|(sym, _)| [sym].into_iter().collect())
                .collect();
        }

        // We have K >= 2, it works with recursive dot lookaheads.
        set.iter_lookaheads()
            .flat_map(|(sym, item)| {
                let source: Array<K, _> = [sym].into_iter().collect();
                self.rec_follow(&source, item)
            })
            .collect()
    }
}

pub type ItemCore<'sid, 'sym, 'rule> = Item<'sid, 'sym, 'rule, 0>;

/// A rule item.
///
/// # Example
/// A -> w • eof
///
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Item<'sid, 'sym, 'rule, const K: usize> {
    pub rule: &'rule Rule<'sid, 'sym>,
    pub position: usize,
    pub lookaheads: Array<K, &'sym Symbol<'sid>>,
}

impl<'sid, 'sym, 'rule, const K: usize> Item<'sid, 'sym, 'rule, K> {
    pub fn dot_lookahead(
        &self,
        rules: &'rule RuleSet<'sid, 'sym>,
    ) -> Vec<(&'sym Symbol<'sid>, ItemCore<'sid, 'sym, 'rule>)> {
        let core = self.into_core();

        if let Some(&sym) = core.symbol().iter().find(|&sym| Symbol::is_terminal(sym)) {
            return vec![(sym, core)];
        }

        let mut set = ItemSet::new([core], []);
        set.close(rules);
        set.iter_lookaheads().collect()
    }
}

impl<const K: usize> Hash for Item<'_, '_, '_, K> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.rule.hash(state);
        self.position.hash(state);
        self.lookaheads.hash(state);
    }
}

impl<'sid, 'sym, 'rule, const K: usize> Item<'sid, 'sym, 'rule, K> {
    fn new(rule: &'rule Rule<'sid, 'sym>, position: usize) -> Option<Self> {
        if rule.rhs.len() >= position {
            Some(Self {
                rule,
                position,
                lookaheads: Default::default(),
            })
        } else {
            None
        }
    }

    /// Turns the item into its core (no lookaheads)
    pub fn into_core(&self) -> ItemCore<'sid, 'sym, 'rule> {
        ItemCore::new(self.rule, self.position).unwrap()
    }

    /// Check if we reached the end of a rule.
    ///
    /// # Example
    /// A -> w • eof
    pub fn is_exhausted(&self) -> bool {
        self.position >= self.rule.rhs.len()
    }

    /// The item is reaching the end of stream (<eos>)
    pub fn is_reaching_end(&self) -> bool {
        self.symbol().map(|sym| sym.eos).unwrap_or(false)
    }

    /// Returns the current symbol.
    /// If A -> w • eof, then returns None.
    pub fn symbol(&self) -> Option<&'sym Symbol<'sid>> {
        self.rule.rhs.get(self.position).copied()
    }

    /// Returns the next rule's item.
    ///
    /// Returns None, if the current rule is exhausted.
    ///
    /// # Example
    /// (A -> • w eof).next() -> (A -> w • eof)
    pub fn next(&self) -> Option<Self> {
        Self::new(self.rule, self.position + 1)
    }
}

/// A set of items.
///
/// The kernel is the original set of items before closure.
/// Items are additional items from closure.
#[derive(Debug, Default)]
pub struct ItemSet<'sid, 'sym, 'rule, const K: usize> {
    // Identifer of the item set.
    pub id: usize,
    kernel: HashSet<Item<'sid, 'sym, 'rule, K>>,
    items: Vec<Item<'sid, 'sym, 'rule, K>>,
}

/// Compares kernel sets.
impl<'sid, 'sym, 'rule, const K: usize> PartialEq for ItemSet<'sid, 'sym, 'rule, K> {
    fn eq(&self, other: &Self) -> bool {
        self.kernel.eq(&other.kernel)
    }
}

impl<'sid, 'sym, 'rule, const K: usize> From<&'rule RuleSet<'sid, 'sym>>
    for ItemSet<'sid, 'sym, 'rule, K>
{
    fn from(value: &'rule RuleSet<'sid, 'sym>) -> Self {
        value
            .iter_rules()
            .next()
            .and_then(|rule| rule.at(0))
            .into_iter()
            .collect()
    }
}

impl<'sid, 'sym, 'rule, const K: usize> FromIterator<Item<'sid, 'sym, 'rule, K>>
    for ItemSet<'sid, 'sym, 'rule, K>
{
    /// Collect the iterator as a kernel set.
    fn from_iter<T: IntoIterator<Item = Item<'sid, 'sym, 'rule, K>>>(iter: T) -> Self {
        Self {
            id: 0,
            kernel: iter.into_iter().collect(),
            items: vec![],
        }
    }
}

impl<'sid, 'sym, 'rule, const K: usize> ItemSet<'sid, 'sym, 'rule, K> {
    pub fn new<I1, I2>(kernel: I1, items: I2) -> Self
    where
        I1: IntoIterator<Item = Item<'sid, 'sym, 'rule, K>>,
        I2: IntoIterator<Item = Item<'sid, 'sym, 'rule, K>>,
    {
        Self {
            id: 0,
            kernel: kernel.into_iter().collect(),
            items: items.into_iter().collect(),
        }
    }

    /// Returns a pair of (terminal, item who consumes it)
    pub fn iter_lookaheads<'a>(
        &'a self,
    ) -> impl Iterator<Item = (&'sym Symbol<'sid>, ItemCore<'sid, 'sym, 'rule>)> + 'a {
        self.iter()
            .flat_map(|i| i.symbol().map(|sym| (sym, i.into_core())))
            .filter(|(s, _)| s.is_terminal())
    }

    pub fn iter_terminal_symbols<'a>(&'a self) -> impl Iterator<Item = &'sym Symbol<'sid>> + 'a {
        self.iter()
            .flat_map(Item::symbol)
            .filter(|&s| s.is_terminal())
    }

    pub fn iter_exhausted_items<'set>(
        &'set self,
    ) -> impl Iterator<Item = &'set Item<'sid, 'sym, 'rule, K>> + 'set {
        self.iter().filter(|item| item.is_exhausted())
    }

    /// Returns true if one of the item is terminating.
    pub fn has_exhausted_items(&self) -> bool {
        self.iter().any(|item| item.is_exhausted())
    }

    pub fn has_item_reaching_eos(&self) -> bool {
        self.iter().any(|item| item.is_reaching_end())
    }

    pub fn get_terminating_rule(&self) -> usize {
        self.iter()
            .find(|item| item.is_exhausted())
            .map(|item| item.rule.id)
            .unwrap()
    }

    /// Execute next for all items within the set.
    pub fn next(&self) -> Self {
        self.iter().flat_map(Item::next).collect()
    }

    /// Iterate over all items within the set.
    pub fn iter(&self) -> impl Iterator<Item = &Item<'sid, 'sym, 'rule, K>> {
        return self.kernel.iter().chain(self.items.iter());
    }

    fn push(&mut self, item: Item<'sid, 'sym, 'rule, K>) {
        if !self.contains(&item) {
            self.items.push(item)
        }
    }

    pub fn contains(&self, item: &Item<'sid, 'sym, 'rule, K>) -> bool {
        self.kernel.contains(item) || self.items.contains(item)
    }

    /// Iterable over all reachable sets from the current set.
    ///
    /// The transition returns the symbol, and the kernel.
    pub fn reachable_sets(&self) -> Vec<(&'sym Symbol<'sid>, ItemSet<'sid, 'sym, 'rule, K>)> {
        self.iter()
            .group_by(|item| item.rule.lhs)
            .into_iter()
            .map(|(sym, items)| (sym, items.flat_map(|item| item.next()).collect()))
            .collect()
    }

    /// Close the item set
    ///
    /// It will fetch all items until the next symbol is a terminal one, or we reach exhaustion.
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use itertools::Itertools;

    use crate::array::Array;
    use crate::{ItemSet, RuleSet, Symbol};

    use crate::fixtures::{fixture_lr0_grammar, fixture_lr1_grammar};

    #[test]
    fn test_001_item_set_closure() {
        let grammar = fixture_lr0_grammar().expect("Cannot generate grammar");
        let rules = RuleSet::new(&grammar);

        let mut set = ItemSet::<0>::from(&rules);
        set.close(&rules);

        let expected_set = ItemSet::new(
            [
                // S → • E eof
                rules.get(0).at(0).unwrap(),
            ],
            [
                // E → • E * B
                rules.get(1).at(0).unwrap(),
                // E → • E + B
                rules.get(2).at(0).unwrap(),
                // E → • B
                rules.get(3).at(0).unwrap(),
                // B → • 0
                rules.get(4).at(0).unwrap(),
                // B → • 1
                rules.get(5).at(0).unwrap(),
            ],
        );

        assert_eq!(set, expected_set)
    }

    #[test]
    fn test_002_first_1_set() {
        let g = fixture_lr1_grammar().expect("cannot create LR(1) grammar");
        let rules = RuleSet::new(&g);
        let values = rules.first::<1>(g.sym("T"));
        let expected_values: HashSet<Array<1, &Symbol<'_>>> = [
            Array::from_iter([g.sym("n")]),
            Array::from_iter([g.sym("+")]),
        ]
        .into_iter()
        .collect();

        println!(
            "{{{}}}",
            rules
                .first::<1>(g.sym("<root>"))
                .iter()
                .map(|a| a.to_string())
                .join(", ")
        );
        assert_eq!(values, expected_values)
    }
}
