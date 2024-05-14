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

#[cfg(test)]
mod tests {
    use crate::{Grammar, GrammarResult, RuleSet};

    fn fixture_grammar() -> GrammarResult<'static, Grammar<'static>> {
        let mut grammar = Grammar::default();

        grammar
            .add_terminal_symbol("0")?
            .add_terminal_symbol("1")?
            .add_terminal_symbol("+")?
            .add_terminal_symbol("*")?
            .add_non_terminal_symbol("E")?
            .add_non_terminal_symbol("B")?
            .add_non_terminal_symbol("S")?;

        grammar
            .add_rule("S", ["E", "<eos>"])?
            .add_rule("E", ["E", "*", "B"])?
            .add_rule("E", ["E", "+", "B"])?
            .add_rule("E", ["B"])?
            .add_rule("B", ["0"])?
            .add_rule("B", ["1"])?;

        Ok(grammar)
    }

    #[test]
    fn test_001_item_set_closure() {
        let grammar = fixture_grammar().expect("Cannot generate grammar");
        let rules = RuleSet::new(&grammar);

        let mut set = RuleItemSet::from(&rules);
        set.close(&rules);

        let expected_set = RuleItemSet::new(
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
}
