use std::{collections::HashSet, hash::Hash};

use itertools::Itertools;

use crate::{array::Array, Rule, RuleSet, Symbol};

pub type ItemSetId = usize;

impl<'sid, 'sym> Rule<'sid, 'sym> {
    pub fn at<'rule, const K: usize>(
        &'rule self,
        position: usize,
    ) -> Option<Item<'sid, 'sym, 'rule, K>> {
        Item::new(self, position)
    }

    pub fn follow<'rule>(
        &'rule self,
        symbol: &'sym Symbol<'sid>,
    ) -> impl Iterator<Item = ItemCore<'sid, 'sym, 'rule>> + 'rule {
        self.rhs
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, &sym)| sym == *symbol)
            //.inspect(|(_, sym)| println!("{}", sym))
            .map(|(pos, _)| self.at::<0>(pos + 1).unwrap())
            //.inspect(|i| println!("{}", i))
            .filter(|i| i.is_exhausted() || i.is_symbol_terminal())
    }
}

impl<'sid, 'sym> RuleSet<'sid, 'sym> {
    pub fn follow(&self, symbol: &'sym Symbol<'sid>) -> HashSet<&'sym Symbol<'sid>> {
        let mut set = HashSet::default();
        let mut visited = HashSet::<&'sym Symbol<'sid>>::default();
        let mut stack = vec![symbol];

        if symbol.is_start() {
            return HashSet::from_iter([self.eos()]);
        }

        while let Some(symbol) = stack.pop() {
            if visited.contains(symbol) {
                continue;
            } else {
                visited.insert(symbol);
            }

            // Follow(X)
            // Get all rules containing X in the rhs list.
            for rule in self.iter().filter(|rule| rule.contains(symbol)) {
                for item in rule.follow(symbol) {
                    // Follow(X, rule) -> {ItemCore...}
                    // If : A → αX•, we add Follow(A) to the Set.
                    if item.is_exhausted() {
                        stack.push(item.rule.lhs);
                    }
                    // A → αX•β
                    else {
                        let subset = self.first(item.symbol().unwrap());
                        set.extend(subset);
                    }
                }
            }
        }

        set
    }

    /// Fetch the terminal symbols from deriving the given non-terminal symbol.
    pub fn first(&self, symbol: &'sym Symbol<'sid>) -> HashSet<&'sym Symbol<'sid>> {
        if symbol.is_terminal() {
            return HashSet::from_iter([symbol]);
        }

        let mut set = HashSet::default();
        let mut visited = HashSet::<&'sym Symbol<'sid>>::default();
        let mut stack = vec![symbol];

        while let Some(symbol) = stack.pop() {
            if visited.contains(symbol) {
                continue;
            } else {
                visited.insert(symbol);
            }

            if symbol.is_terminal() {
                set.insert(symbol);
                continue;
            }

            for rule in self.iter_by_symbol(symbol) {
                let symbol = *rule.rhs.first().unwrap();
                stack.push(symbol);
            }
        }

        set
    }

    /// Returns the start item set (#0)
    ///
    /// # Panics
    /// Panics if there are no start rule (#0), or the start rule is empty.
    pub fn start_item_set<'rule, const K: usize>(&'rule self) -> ItemSet<'sid, 'sym, 'rule, K> {
        let mut start = self
            .get(0)
            .at::<K>(0)
            .unwrap();
        
        if K > 0 {
            start.lookaheads = Array::from_iter([self.eos()]);
        }

        [start].into_iter().collect()
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

impl<const K: usize> std::fmt::Display for Item<'_, '_, '_, K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut rhs = self
            .rule
            .rhs
            .iter()
            .map(ToString::to_string)
            .enumerate()
            .map(|(pos, mut s)| {
                if pos == self.position {
                    s.insert_str(0, "• ");
                }
                s
            })
            .join(" ");
        
        if self.is_exhausted() {
            rhs.push_str(" •")
        }

        write!(f, "{}. {} -> {}", self.rule.id, self.rule.lhs, rhs)?;

        if !self.lookaheads.is_empty() {
            write!(f, ", {}", self.lookaheads)?;
        }

        Ok(())
    }
}

impl<'sid, 'sym, 'rule, const K: usize> Item<'sid, 'sym, 'rule, K> {
    pub fn follow(&self, rules: &'rule RuleSet<'sid, 'sym>) -> HashSet<&'sym Symbol<'sid>> {
        self.symbol()
            .map(|sym| rules.follow(sym))
            .unwrap_or_default()
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
    /// Creates a new rule
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
    /// A -> w •
    pub fn is_exhausted(&self) -> bool {
        self.position >= self.rule.rhs.len()
    }

    /// The item is reaching the end of stream (<eos>)
    pub fn is_reaching_end(&self) -> bool {
        self.symbol().map(|sym| sym.is_eos()).unwrap_or(false)
    }

    pub fn is_symbol_non_terminal(&self) -> bool {
        self.symbol()
            .map(|symbol| !symbol.is_terminal())
            .unwrap_or(false)    
    }
    /// The item is reaching immediately a terminal symbol
    pub fn is_symbol_terminal(&self) -> bool {
        self.symbol()
            .map(|symbol| symbol.is_terminal())
            .unwrap_or(false)
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

impl<'sid, 'sym, 'rule, const K: usize> std::fmt::Display for ItemSet<'sid, 'sym, 'rule, K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}{{", self.id)?;
        for item in self.iter() {
            write!(f, "{},", item)?;
        }
        write!(f, "}}")
    }
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
            .iter()
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

    pub fn iter_terminal_symbols<'a>(&'a self) -> impl Iterator<Item = &'sym Symbol<'sid>> + 'a {
        self.iter()
            .flat_map(Item::symbol)
            .filter(|&s| s.is_terminal())
    }

    pub fn iter_immediate_terminal_items<'set>(
        &'set self,
    ) -> impl Iterator<Item = &'set Item<'sid, 'sym, 'rule, K>> + 'set {
        self.iter().filter(|item| item.is_symbol_terminal())
    }

    /// Iterate over all exhausted items (A -> w •)
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

    pub fn get_exhausted_rule(&self) -> usize {
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
    pub fn reachable_sets(&self, rules: &'rule RuleSet<'sid, 'sym>) -> Vec<(&'sym Symbol<'sid>, ItemSet<'sid, 'sym, 'rule, K>)> {
        rules
            .iter_symbols()
            .filter(|sym| !(sym.is_eos() || sym.is_epsilon()))
            .map(|sym| (
                sym, 
                ItemSet::from_iter(
                    self.iter()
                        .filter(|item| item.symbol() == Some(sym))
                        .cloned()
                )
            ))
            .map(|(sym, set)| (sym, set.next()))
            .filter(|(_, set)| !set.kernel.is_empty())
            .collect()

    }

    /// This methods is the union of all follow sets of all items which is followed by the given symbol.
    pub fn follow(
        &self,
        symbol: &'sym Symbol<'sid>,
        rules: &'rule RuleSet<'sid, 'sym>,
    ) -> HashSet<&'sym Symbol<'sid>> {
        if symbol == rules.start() {
            return HashSet::from_iter([rules.eos()]);
        }
        self.iter()
            .filter(|item| item.symbol() == Some(symbol))
            .flat_map(|item| item.follow(rules))
            .collect()
    }

    /// Add lookaheads to the items.  
    /// 
    /// TODO : Can be improved with cached follow sets.
    pub fn add_lookaheads(&mut self, rules: &'rule RuleSet<'sid, 'sym>) {
        let mut items = Vec::<Item<'sid, 'sym, 'rule, K>>::default();

        for item in self.items.iter() {
            for symbol in rules.follow(item.rule.lhs) {
                let mut item = item.clone();
                item.lookaheads = [symbol].into_iter().collect();
                items.push(item);
            }
        }

        self.items = items;
    }

    /// Close the item set
    ///
    /// It will fetch all items until the next symbol is a terminal one, or we reach exhaustion.
    pub fn close(&mut self, rules: &'rule RuleSet<'sid, 'sym>) {
        let mut stack: Vec<_> = self.kernel.clone().into_iter().collect();

        while let Some(item) = stack.pop() {
            if item.is_symbol_non_terminal() {
                let sym = item.symbol().unwrap();
                for item in rules.iter_by_symbol(sym).flat_map(|rule| rule.at(0)) {
                    if !self.contains(&item) {
                        stack.push(item.clone());
                        self.push(item);
                    }
                }
            }
        }

        if K == 1 {
            self.add_lookaheads(rules);
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::{ItemSet, RuleSet, Symbol};
    use std::collections::HashSet;

    use crate::fixtures::{fixture_lr0_grammar, fixture_lr1_grammar};

    #[test]
    fn test_001_item_set_closure() {
        let grammar = fixture_lr0_grammar().expect("Cannot generate grammar");
        let rules = RuleSet::new(&grammar);

        let mut set = rules.start_item_set::<0>();
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
    fn test_002_first_set() {
        let g = fixture_lr1_grammar().expect("cannot create LR(1) grammar");
        let rules = RuleSet::new(&g);

        let mut values = rules.first(g.sym("T"));
        let mut expected_values: HashSet<&Symbol<'_>> =
            HashSet::from_iter([g.sym("n"), g.sym("+")]);
        assert_eq!(values, expected_values);

        values = rules.first(g.sym("E"));
        expected_values = HashSet::from_iter([g.sym("n"), g.sym("("), g.sym("+")]);
        assert_eq!(values, expected_values);

        values = rules.first(g.start());
        expected_values = HashSet::from_iter([g.sym("n"), g.sym("("), g.sym("+")]);
        assert_eq!(values, expected_values);
    }

    #[test]
    /// Follow(A)
    fn test_003_follow_set() {
        let g = fixture_lr1_grammar().expect("cannot create LR(1) grammar");
        let rules = RuleSet::new(&g);

        let values = rules.follow(g.start());
        let expected_values = HashSet::from_iter([g.eos()]);
        assert_eq!(values, expected_values);

        let values = rules.follow(g.sym("T"));
        let expected_values = HashSet::from_iter([g.sym(")"), g.sym("+"), g.eos()]);
        assert_eq!(values, expected_values);
    }

    #[test]
    /// Follow(In, A)
    fn test_004_item_set_follow_set() {
        let g = fixture_lr1_grammar().expect("cannot create LR(1) grammar");
        let rules = RuleSet::new(&g);
        let mut i0 = rules.start_item_set::<0>();
        i0.close(&rules);

        let mut values = i0.follow(g.start(), &rules);
        let mut expected_values = HashSet::from_iter([g.eos()]);
        assert_eq!(values, expected_values);

        values = i0.follow(g.sym("E"), &rules);
        expected_values = HashSet::from_iter([g.eos(), g.sym(")")]);
        assert_eq!(values, expected_values);

        values = i0.follow(g.sym("T"), &rules);
        expected_values = HashSet::from_iter([g.eos(), g.sym(")"), g.sym("+")]);
        assert_eq!(values, expected_values);
    }
}
