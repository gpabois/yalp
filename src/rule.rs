use std::hash::Hash;

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
    pub rhs: Vec<&'sid str>,
}

impl<'sid> RuleDef<'sid> {
    pub fn new<I>(id: RuleId, lhs: &'sid str, rhs: I) -> Self
    where
        I: IntoIterator<Item = &'sid str>,
    {
        Self {
            id,
            lhs,
            rhs: rhs.into_iter().collect(),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
/// A grammar rule
///
/// # Example
/// A -> w eof
pub struct Rule<'sid, 'sym> {
    pub id: RuleId,
    pub lhs: &'sym Symbol<'sid>,
    pub rhs: Vec<&'sym Symbol<'sid>>,
}

impl Hash for Rule<'_, '_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.lhs.hash(state);
        self.rhs.hash(state);
    }
}

/// A set of rules.
#[derive(Debug, Default)]
pub struct RuleSet<'sid, 'sym>(Vec<Rule<'sid, 'sym>>);

impl<'sid, 'sym> RuleSet<'sid, 'sym> {
    pub fn new(grammar: &'sym Grammar<'sid>) -> Self {
        Self(grammar.iter_rules().collect())
    }

    /// Iterate over all rules of the grammar
    pub fn iter_rules(&self) -> impl Iterator<Item = &Rule<'sid, 'sym>> {
        self.0.iter()
    }

    pub fn iter_symbol_related_rules(
        &self,
        sym: &'sym Symbol<'sid>,
    ) -> impl Iterator<Item = &Rule<'sid, 'sym>> {
        self.iter_rules().filter(|rule| *rule.lhs == *sym)
    }

    pub fn get(&self, id: RuleId) -> &Rule<'sid, 'sym> {
        self.iter_rules().find(|rule| rule.id == id).unwrap()
    }
}