use itertools::Itertools;
use yalp_shared::symbol::{Symbol, SymbolName};

use std::borrow::{Borrow, Cow};
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

use pb_bnf::syntax::BnfSyntax;

pub type RuleId = usize;
pub type StaticSymbol = SymbolName<'static>;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Syntax<'syntax>(Cow<'syntax, [Rule<'syntax>]>);

impl<'syntax> Syntax<'syntax> {
    /// Iterate the rules behind a specific non-terminal symbols.
    pub fn iter_rules_by_symbol<'a>(
        &'a self,
        sym: &'a str,
    ) -> impl Iterator<Item = &'a Rule<'syntax>> {
        self.as_ref().iter().filter(move |rule| rule.lhs.is(sym))
    }

    /// Iterate over all symbols used in the syntax.
    pub fn iter_symbols(&self) -> impl Iterator<Item = &SymbolName<'syntax>> {
        self.as_ref()
            .iter()
            .flat_map(|rule| std::iter::once(&rule.lhs).chain(rule.rhs.as_ref().iter()))
            .dedup()
    }
}

impl<'syntax> FromIterator<Rule<'syntax>> for Syntax<'syntax> {
    fn from_iter<T: IntoIterator<Item = Rule<'syntax>>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<'syntax> AsRef<[Rule<'syntax>]> for Syntax<'syntax> {
    fn as_ref(&self) -> &[Rule<'syntax>] {
        self.0.borrow()
    }
}

impl<'syntax> AsMut<Vec<Rule<'syntax>>> for Syntax<'syntax> {
    fn as_mut(&mut self) -> &mut Vec<Rule<'syntax>> {
        self.0.to_mut()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule<'syntax> {
    pub lhs: SymbolName<'syntax>,
    pub rhs: Definition<'syntax>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Definition<'syntax>(Cow<'syntax, [SymbolName<'syntax>]>);

impl<'syntax> AsRef<[SymbolName<'syntax>]> for Definition<'syntax> {
    fn as_ref(&self) -> &[SymbolName<'syntax>] {
        self.0.borrow()
    }
}

impl<'syntax> AsMut<Vec<SymbolName<'syntax>>> for Definition<'syntax> {
    fn as_mut(&mut self) -> &mut Vec<SymbolName<'syntax>> {
        self.0.to_mut()
    }
}

impl<'syntax> FromIterator<SymbolName<'syntax>> for Definition<'syntax> {
    fn from_iter<T: IntoIterator<Item = SymbolName<'syntax>>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<'syntax> From<BnfSyntax<'syntax>> for Syntax<'syntax> {
    fn from(value: BnfSyntax<'syntax>) -> Self {
        let mut syntax = Self::default();

        value.iter().cloned().enumerate().for_each(|(i, rule)| {
            // if root = A0...An, then root = B and B = A0...An
            if i == 0 && rule.rhs().len() > 1 {
                syntax.push(Rule {
                    lhs: SymbolName::from("root"),
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

    pub fn into_term(&self, symbol: &'syntax SymbolName) -> PrepSymbol<'syntax> {
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

impl<'syntax> From<&Syntax<'syntax>> for PrepSyntax<'syntax> {
    fn from(syntax: &Syntax<'syntax>) -> Self {
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
    Terminal(&'a SymbolName),
    NonTerminal(&'a SymbolName),
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

    pub fn is(&self, symbol: &SymbolName) -> bool {
        match self {
            PrepSymbol::Terminal(sym) => *sym == symbol,
            PrepSymbol::NonTerminal(sym) => *sym == symbol,
            PrepSymbol::EOS => false,
        }
    }
}

#[derive(Default, Clone)]
pub struct SymbolSet<'syntax> {
    pub terminals: HashSet<SymbolName<'syntax>>,
    pub non_terminals: HashSet<SymbolName<'syntax>>,
    pub start: Option<SymbolName<'syntax>>,
}

impl<'syntax> SymbolSet<'syntax> {
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = PrepSymbol<'syntax>> + 'a {
        self.terminals
            .iter()
            .copied()
            .map(Symbol::Terminal)
            .chain(self.non_terminals.iter().copied().map(Symbol::NonTerminal))
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
