use std::{hash::Hash, vec::Drain};

use itertools::Itertools;

use crate::{grammar::traits::Grammar, traits::IntoRef};

use super::Symbol;

/// The rule's identifier in the grammar.
pub type RuleId = usize;

/// An iterator over all RHS nodes.
pub type AstIter<'a, Ast> = Drain<'a, Ast>;

/// A rule reducer
pub type RuleReducer<'b, Ast> = for<'a, 'c, 'd> fn(&'a Rule<'b, 'c>, AstIter<'d, Ast>) -> Ast;

#[derive(Debug, Eq, PartialEq)]
/// A grammar rule
///
/// This object is produced by the grammar with
/// references to the symbols.
///
/// # Example
/// A -> w <eos>
pub struct Rule<'sid, 'sym> {
    pub id: RuleId,
    pub lhs: &'sym Symbol<'sid>,
    pub rhs: Vec<&'sym Symbol<'sid>>,
}

impl std::fmt::Display for Rule<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}) {} => {}",
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
    /// Check the rules contains a certain symbol in its RHS.
    #[inline(always)]
    pub fn contains(&self, sym: &'sym Symbol<'sid>) -> bool {
        self.rhs.contains(&sym)
    }
}

/// A set of rules.
///
/// This object is used to generate parser tables.
#[derive(Debug)]
pub struct RuleSet<'sid, 'sym>(Vec<Rule<'sid, 'sym>>, &'sym [Symbol<'sid>]);

impl<'sid, 'sym, 'a> IntoRef<'sym, [Symbol<'sid>]> for &'a RuleSet<'sid, 'sym> {
    fn into_ref(self) -> &'sym [Symbol<'sid>] {
        self.1
    }
}

impl<'sid, 'sym> RuleSet<'sid, 'sym> {
    pub fn new<G>(grammar: &'sym G) -> Self
    where
        G: Grammar<'sid, 'sym>,
        &'sym G: IntoRef<'sym, [Symbol<'sid>]>,
    {
        Self(grammar.iter_rules().collect(), grammar.as_symbol_slice())
    }

    pub fn iter_symbols(&self) -> impl Iterator<Item = &'sym Symbol<'sid>> {
        self.1.iter()
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
}

/// Defines a grammar rule
///
/// This method is internal to the grammar object.
/// The grammar will generate the Rule object with references to reduce
/// the in-memory print.
/// X := A1..An
#[derive(Debug, PartialEq)]
pub struct RuleDef<'sid> {
    pub lhs: &'sid str,
    pub rhs: &'sid [&'sid str],
}

#[macro_export]
macro_rules! rule {
    ($lhs:expr => $($rhs:expr)*) => {
        $crate::RuleDef::new(
            $lhs,
            &[$($rhs),*]
        )
    };

}

impl<'sid> RuleDef<'sid> {
    pub const fn new(lhs: &'sid str, rhs: &'sid [&'sid str]) -> Self {
        Self { lhs, rhs }
    }
}
