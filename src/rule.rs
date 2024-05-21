use std::hash::Hash;

use itertools::Itertools;

use crate::Grammar;

use super::Symbol;

pub type RuleId = usize;

/// Defines a grammar rule
///
/// X := A1..An
#[derive(Debug, PartialEq)]
pub(crate) struct RuleDef<'sid> {
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

impl std::fmt::Display for Rule<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}. {} -> {}",
            self.id,
            self.lhs,
            self.rhs.iter().map(|s| s.to_string()).join(" ")
        )
    }
}

impl Hash for Rule<'_, '_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.lhs.hash(state);
        self.rhs.hash(state);
    }
}

impl<'sid, 'sym> Rule<'sid, 'sym> {
    #[inline(always)]
    pub fn contains(&self, sym: &'sym Symbol<'sid>) -> bool {
        self.rhs.contains(&sym)
    }
}

/// A set of rules.
#[derive(Debug)]
pub struct RuleSet<'sid, 'sym>(Vec<Rule<'sid, 'sym>>, &'sym Grammar<'sid>);

impl<'sid, 'sym> RuleSet<'sid, 'sym> {
    pub fn new(grammar: &'sym Grammar<'sid>) -> Self {
        Self(grammar.iter_rules().collect(), grammar)
    }

    pub fn iter_symbols(&self)-> impl Iterator<Item = &'sym Symbol<'sid>> {
        self.1.iter_terminal_symbols().chain(self.1.iter_non_terminal_symbols())
    }

    /// Iterate over all rules of the grammar
    pub fn iter(&self) -> impl Iterator<Item = &Rule<'sid, 'sym>> {
        self.0.iter()
    }

    pub fn iter_by_symbol(
        &self,
        sym: &'sym Symbol<'sid>,
    ) -> impl Iterator<Item = &Rule<'sid, 'sym>> {
        self.iter().filter(|rule| *rule.lhs == *sym)
    }

    pub fn get(&self, id: RuleId) -> &Rule<'sid, 'sym> {
        self.iter().find(|rule| rule.id == id).unwrap()
    }

    pub fn start(&self) -> &'sym Symbol<'sid> {
        self.1.start()
    }

    pub fn eos(&self) -> &'sym Symbol<'sid> {
        self.1.eos()
    }

    pub fn epsilon(&self) -> &'sym Symbol<'sid> {
        self.1.epsilon()
    }
}

