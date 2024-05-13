use crate::Grammar;

use super::Symbol;

pub type RuleId = usize;

/// Defines a grammar rule
/// 
/// X := A1..An
#[derive(Debug, PartialEq)]
pub struct RuleDef<'sid> {
    /// Identifier of the rule
    pub id: RuleId,
    pub lhs: &'sid str,
    pub rhs: Vec<&'sid str>
}

impl<'sid> RuleDef<'sid> {
    pub fn new<I>(id: RuleId, lhs: &'sid str, rhs: I) -> Self where I: IntoIterator<Item=&'sid str> {
        Self {
            id, lhs, rhs: rhs.into_iter().collect()
        }
    }
}

#[derive(Debug, PartialEq)]
/// A grammar rule
/// 
/// # Example
/// A -> w eof
pub struct Rule<'sid, 'sym> {
    pub id: RuleId,
    pub lhs: &'sym Symbol<'sid>,
    pub rhs: Vec<&'sym Symbol<'sid>>
}

impl<'sid, 'sym> Rule<'sid, 'sym> {
    pub fn at<'rule>(&'rule self, position: usize) -> Option<RuleItem<'sid, 'sym, 'rule>> {
        RuleItem::new(self, position)
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
/// A rule item.
/// 
/// # Example 
/// A -> w • eof
pub struct RuleItem<'sid, 'sym, 'rule> {
    pub rule: &'rule Rule<'sid, 'sym>,
    pub position: usize
}

impl<'sid, 'sym, 'rule> RuleItem<'sid, 'sym, 'rule> {
    fn new(rule: &'rule Rule<'sid, 'sym>, position: usize) -> Option<Self> {
        if rule.rhs.len() >= position {
            return Some(Self{rule, position})
        } else {
            None
        }
    }

    /// Check if we reached the end of a rule.
    /// 
    /// # Example
    /// A -> w • eof
    pub fn is_terminating(&self) -> bool {
        return self.position >= self.rule.rhs.len()
    }
    
    /// Returns the current symbol.
    /// If A -> w • eof, then returns None.
    pub fn symbol(&self) -> Option<&'sym Symbol<'sid>> {
        self.rule.rhs.get(self.position).copied()
    }

    /// Returns the next rule's item.
    /// 
    /// # Example
    /// (A -> • w  eof).next() -> (A -> w • eof)
    pub fn next(&self) -> Option<Self> {
        Self::new(self.rule, self.position + 1)
    }
}

pub struct RuleSet<'sid, 'sym>(Vec<Rule<'sid, 'sym>>);

impl<'sid, 'sym> RuleSet<'sid, 'sym> {
    pub fn new(grammar: &'sym Grammar<'sid>) -> Self {
        Self(grammar.iter_rules().collect())
    }

    /// Iterate over all rules of the grammar
    pub fn iter_rules(&self) -> impl Iterator<Item=&Rule<'sid, 'sym>> {
        self.0.iter()
    }

    pub fn iter_symbol_related_rules(&self, sym: &'sym Symbol<'sid>) -> impl Iterator<Item=&Rule<'sid, 'sym>> {
        self
            .iter_rules()
            .filter(|rule| *rule.lhs == *sym)
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct RuleItemSet<'sid, 'sym, 'rule> {
    items: Vec<RuleItem<'sid, 'sym, 'rule>>
}

impl<'sid, 'sym, 'rule> FromIterator<RuleItem<'sid, 'sym, 'rule>> for RuleItemSet<'sid, 'sym, 'rule> {
    fn from_iter<T: IntoIterator<Item = RuleItem<'sid, 'sym, 'rule>>>(iter: T) -> Self {
        let mut set = Self::default();
        set.append(iter.into_iter());
        set
    }
}

impl<'sid, 'sym, 'rule> RuleItemSet<'sid, 'sym, 'rule> 
{
    /// Returns true if one of the item is terminating.
    pub fn has_terminating_item(&self) -> bool {
        self.iter()
            .find(|item| item.is_terminating())
            .is_some()
    }

    pub fn next(&self) -> Self {
        self.iter().flat_map(RuleItem::next).collect()
    }

    pub fn iter(&self) -> impl Iterator<Item=&RuleItem<'sid, 'sym, 'rule>> {
        return self.items.iter()
    }

    pub fn push(&mut self, item: RuleItem<'sid, 'sym, 'rule>) {
        if !self.contains(&item) {
            self.items.push(item)
        }
    }

    pub fn contains(&self, item: &RuleItem<'sid, 'sym, 'rule>) -> bool {
        self.items.contains(item)
    }

    pub fn pop(&mut self) -> Option<RuleItem<'sid, 'sym, 'rule>> {
        self.items.pop()
    }

    pub fn append<I>(&mut self, items: I) where I: Iterator<Item=RuleItem<'sid, 'sym, 'rule>>
    {
        for item in items {
            self.push(item)
        }
    }

    /// Close the item set
    /// 
    /// It will fetch all rules until the next symbol is a terminal one, or we reach the end of a rule.
    pub fn close(&mut self, rules: &'rule RuleSet<'sid, 'sym>) {
        let mut stack = self.items.clone();

        while let Some(item) = stack.pop() {
            if item.symbol().map(|sym| !sym.terminal).unwrap_or(false) {
                let sym = item.symbol().unwrap();
                for item in rules.iter_symbol_related_rules(sym).flat_map(|rule| rule.at(0)) {
                    if !self.contains(&item) {
                        stack.push(item);
                        self.push(item);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Grammar, GrammarResult, RuleItem, RuleSet};

    fn fixture_grammar() -> GrammarResult<'static, Grammar<'static>> {
        let mut grammar = Grammar::default();

        grammar
            .add_terminal_symbol("0")?
            .add_terminal_symbol("1")?
            .add_terminal_symbol("+")?
            .add_terminal_symbol("*")?
            .add_non_terminal_symbol("E")?
            .add_non_terminal_symbol("B")?;
    
        grammar
            .add_rule("E", ["E", "*", "B"])?
            .add_rule("E", ["E", "+", "B"])?
            .add_rule("E", ["B"])?
            .add_rule("B", ["0"])?
            .add_rule("B", ["1"])?;

        Ok(grammar)
    }

    #[test]
    fn test_001_closure() {
        let grammar = fixture_grammar().expect("Cannot generate grammar");
        let rules = RuleSet::new(&grammar);

    }
}