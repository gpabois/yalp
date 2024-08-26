use itertools::Itertools;

use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

use pb_bnf::prelude::*;
pub use pb_bnf::symbol::{Symbol, SymbolRef};
use pb_bnf::syntax::Syntax as BnfSyntax;

pub type RuleId = usize;
pub type StaticSymbol = SymbolRef<'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyntaxRef<'a>(&'a [RuleRef<'a>]);

impl<'a> SyntaxRef<'a> {
    pub const fn new(rules: &'a [RuleRef<'a>]) -> Self {
        Self(rules)
    }

    pub fn to_owned(&self) -> Syntax {
        self.iter().map(|rule| rule.to_owned()).collect()
    }
}

pub type StaticSyntax = SyntaxRef<'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuleRef<'a> {
    lhs: SymbolRef<'a>,
    rhs: DefinitionRef<'a>,
}

pub type StaticRule = RuleRef<'static>;

impl<'a> RuleRef<'a> {
    pub const fn new(lhs: SymbolRef<'a>, rhs: &'a [SymbolRef<'a>]) -> Self {
        Self {
            lhs,
            rhs: DefinitionRef::new(rhs),
        }
    }

    pub fn to_owned(&self) -> Rule {
        Rule {
            lhs: self.lhs.to_owned(),
            rhs: self.rhs.to_owned(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DefinitionRef<'a>(&'a [SymbolRef<'a>]);

impl<'a> DefinitionRef<'a> {
    pub const fn new(terms: &'a [SymbolRef<'a>]) -> Self {
        Self(terms)
    }

    pub fn to_owned(&self) -> Definition {
        self.iter().map(|sym| sym.to_owned()).collect()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Syntax(Vec<Rule>);

impl Syntax {
    /// Iterate the rules behind a specific non-terminal symbols.
    pub fn iter_rules_by_symbol<'a>(&'a self, sym: &'a Symbol) -> impl Iterator<Item = &'a Rule> {
        self.iter().filter(move |rule| &rule.lhs == sym)
    }

    /// Iterate over all symbols used in the syntax.
    pub fn iter_symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.iter()
            .flat_map(|rule| std::iter::once(&rule.lhs).chain(rule.rhs.iter()))
            .dedup()
    }
}

impl FromIterator<Rule> for Syntax {
    fn from_iter<T: IntoIterator<Item = Rule>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl Deref for Syntax {
    type Target = Vec<Rule>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Syntax {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub lhs: Symbol,
    pub rhs: Definition,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Definition(Vec<Symbol>);

impl FromIterator<Symbol> for Definition {
    fn from_iter<T: IntoIterator<Item = Symbol>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl Deref for Definition {
    type Target = Vec<Symbol>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Definition {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<BnfSyntax> for Syntax {
    fn from(value: BnfSyntax) -> Self {
        let mut syntax = Self::default();

        value.iter().cloned().enumerate().for_each(|(i, rule)| {
            // if root = A0...An, then root = B and B = A0...An
            if i == 0 && rule.rhs().len() > 1 {
                syntax.push(Rule {
                    lhs: Symbol::from("root"),
                    rhs: Definition::from_iter([rule.lhs().clone()]),
                });
            }

            rule.rhs().iter().for_each(|def| {
                syntax.push(Rule {
                    lhs: rule.lhs().clone(),
                    rhs: def.iter().cloned().map(|term| term.into_symbol()).collect(),
                })
            });
        });

        syntax
    }
}

/// Preprocessed syntax for parsing generation
pub struct PrepSyntax<'syntax> {
    pub symbols: SymbolSet<'syntax>,
    pub rules: Vec<PrepRule<'syntax>>,
}

impl<'syntax> PrepSyntax<'syntax> {
    pub fn start(&self) -> Option<PrepSymbol<'syntax>> {
        self.symbols.start.map(PrepSymbol::NonTerminal)
    }

    pub fn into_term(&self, symbol: &'syntax Symbol) -> PrepSymbol<'syntax> {
        if self.symbols.terminals.contains(symbol) {
            PrepSymbol::Terminal(symbol)
        } else {
            PrepSymbol::NonTerminal(symbol)
        }
    }

    pub fn sym(&self, id: &str) -> Option<PrepSymbol<'syntax>> {
        self.symbols.iter().find(|sym| sym.is(id))
    }
}

impl<'a> From<&'a Syntax> for PrepSyntax<'a> {
    fn from(syntax: &'a Syntax) -> Self {
        let symbols = SymbolSet::from(syntax);
        let rules = syntax.iter().enumerate().map(|(id, rule)| {
            let lhs = PrepSymbol::NonTerminal(&rule.lhs);
            let mut rhs = rule
                .rhs
                .iter()
                .map(|sym| {
                    if symbols.terminals.contains(sym) {
                        PrepSymbol::Terminal(sym)
                    } else {
                        PrepSymbol::NonTerminal(sym)
                    }
                })
                .collect::<PrepDefinition>();

            // root rule, add <eos>
            if id == 0 {
                rhs.push(PrepSymbol::EOS);
            }
        });

        Self { symbols, rules }
    }
}

/// Preprocessed syntax rule for parsing generation
pub struct PrepRule<'a> {
    pub lhs: PrepSymbol<'a>,
    pub rhs: PrepDefinition<'a>,
}

/// Preprocessed rule definition for parsing generation
pub struct PrepDefinition<'a>(Vec<PrepSymbol<'a>>);

impl<'a> Deref for PrepDefinition<'a> {
    type Target = Vec<PrepSymbol<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for PrepDefinition<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> FromIterator<PrepSymbol<'a>> for PrepDefinition<'a> {
    fn from_iter<T: IntoIterator<Item = PrepSymbol<'a>>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Preprocess rule definition term for parsing generation
pub enum PrepSymbol<'a> {
    Terminal(&'a Symbol),
    NonTerminal(&'a Symbol),
    EOS,
}

impl PrepSymbol<'_> {
    pub fn is_eos(&self) -> bool {
        matches!(self, Self::EOS)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Terminal(_))
    }

    pub fn is_non_terminal(&self) -> bool {
        matches!(self, Self::NonTerminal(_))
    }

    pub fn is(&self, symbol: &Symbol) -> bool {
        match self {
            PrepSymbol::Terminal(sym) => *sym == symbol,
            PrepSymbol::NonTerminal(sym) => *sym == symbol,
            PrepSymbol::EOS => false,
        }
    }
}

#[derive(Default, Clone)]
pub struct SymbolSet<'syntax> {
    pub terminals: HashSet<&'syntax Symbol>,
    pub non_terminals: HashSet<&'syntax Symbol>,
    pub start: Option<&'syntax Symbol>,
}

impl<'syntax> SymbolSet<'syntax> {
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = PrepSymbol<'syntax>> + 'a {
        self.terminals
            .iter()
            .copied()
            .map(PrepSymbol::Terminal)
            .chain(
                self.non_terminals
                    .iter()
                    .copied()
                    .map(PrepSymbol::NonTerminal),
            )
    }
}

impl<'a> From<&'a Syntax> for SymbolSet<'a> {
    fn from(syntax: &'a Syntax) -> Self {
        let mut set = SymbolSet::default();

        syntax.iter_symbols().for_each(|sym| {
            if syntax.iter_rules_by_symbol(sym).any(|_| true) {
                set.non_terminals.insert(sym)
            } else {
                set.terminals.insert(sym)
            }
        });

        set.start = syntax.iter_symbols().next();

        set
    }
}
