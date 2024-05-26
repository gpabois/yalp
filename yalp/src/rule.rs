use std::{hash::Hash, vec::Drain};

use itertools::Itertools;

use crate::{grammar::traits::Grammar, YalpError};

use super::Symbol;

/// The rule's identifier in the grammar.
pub type RuleId = usize;

/// An iterator over all RHS nodes.
pub type AstIter<'a, Ast> = Drain<'a, Ast>;

/// A rule reducer
pub type RuleReducer<'b, Ast, CustomError> =
    for<'a, 'c> fn(&'a Rule<'b>, AstIter<'c, Ast>) -> Result<Ast, YalpError<CustomError>>;

pub mod traits {
    use crate::RuleDef;

    pub trait RuleDefSlice<'sid>: AsRef<[RuleDef<'sid>]> {
        fn as_rule_def_slice(&self) -> &[RuleDef<'sid>];
    }

    impl<'sid, T> RuleDefSlice<'sid> for T
    where
        T: AsRef<[RuleDef<'sid>]>,
    {
        fn as_rule_def_slice(&self) -> &[RuleDef<'sid>] {
            self.as_ref()
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
/// A grammar rule
///
/// This object is produced by the grammar with
/// references to the symbols.
///
/// # Example
/// A -> w <eos>
pub struct Rule<'sid> {
    pub id: RuleId,
    pub lhs: Symbol<'sid>,
    pub rhs: Vec<Symbol<'sid>>,
}

impl std::fmt::Display for Rule<'_> {
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

impl Hash for Rule<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.lhs.hash(state);
        self.rhs.hash(state);
    }
}

impl Rule<'_> {
    /// Check the rules contains a certain symbol in its RHS.
    #[inline(always)]
    pub fn contains(&self, sym: &Symbol<'_>) -> bool {
        self.rhs.contains(sym)
    }
}

/// A set of rules.
///
/// This object is used to generate parser tables.
#[derive(Debug)]
pub struct RuleSet<'sid, 'sym>(Vec<Rule<'sid>>, &'sym [Symbol<'sid>]);

impl<'sid, 'sym> AsRef<[Symbol<'sid>]> for RuleSet<'sid, 'sym> {
    fn as_ref(&self) -> &[Symbol<'sid>] {
        self.1
    }
}

impl<'sid, 'sym> RuleSet<'sid, 'sym> {
    pub fn new<G>(grammar: &'sym G) -> Self
    where
        G: Grammar<'sid>,
    {
        Self(grammar.iter_rules().collect(), grammar.as_symbol_slice())
    }

    pub fn iter_symbols<'a>(&'a self) -> impl Iterator<Item = Symbol<'sid>> + 'a
    where
        'sid: 'a,
    {
        self.1.iter().copied()
    }

    /// Iterate over all rules of the grammar
    pub fn iter(&self) -> impl Iterator<Item = &Rule<'sid>> {
        self.0.iter()
    }

    pub fn iter_by_symbol<'a>(
        &'a self,
        sym: &Symbol<'sid>,
    ) -> impl Iterator<Item = &Rule<'sid>> + 'a
    where
        'sid: 'a,
    {
        let sym = *sym;
        self.iter().filter(move |rule| rule.lhs == sym)
    }

    pub fn borrow_rule(&self, id: RuleId) -> &Rule<'sid> {
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

impl<'sid> RuleDef<'sid> {
    pub const fn new(lhs: &'sid str, rhs: &'sid [&'sid str]) -> Self {
        Self { lhs, rhs }
    }
}
