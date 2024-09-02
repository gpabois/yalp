use std::borrow::Cow;

use itertools::Itertools;

use crate::{
    prelude::*,
    symbol::{Symbol, SymbolId, SymbolSet},
};

pub struct KernelizeSyntax;

/// Finalize the syntax to generate parsers.
pub struct FinalizeSyntax<'syntax> {
    pub set: SymbolSet<'syntax>,
}

pub type SyntaxKernel<'syntax> = Syntax<'syntax, DefinitionKernel<'syntax>>;
pub type RuleKernel<'syntax> = Rule<'syntax, DefinitionKernel<'syntax>>;
pub type DefinitionKernel<'syntax> = Definition<'syntax, SymbolId<'syntax>>;

pub type FinalizedSyntax<'syntax> = Syntax<'syntax, FinalizedDefinition<'syntax>>;
pub type FinalizedRule<'syntax> = Rule<'syntax, FinalizedDefinition<'syntax>>;
pub type FinalizedDefinition<'syntax> = Definition<'syntax, Symbol<'syntax>>;

impl<'syntax> TransformSyntax<'syntax, FinalizeSyntax<'syntax>> for SyntaxKernel<'syntax> {
    type Transformed = FinalizedSyntax<'syntax>;

    fn transform_syntax(self, ctx: &mut FinalizeSyntax<'syntax>) -> Self::Transformed {
        self.into_iter()
            .enumerate()
            .map(|(rule_id, rule)| {
                let mut rule = FinalizedRule {
                    lhs: rule.lhs,
                    rhs: rule
                        .rhs
                        .into_iter()
                        .map(|term| ctx.set[term].clone())
                        .collect(),
                };

                if rule_id == 0 {
                    rule.rhs.as_mut().push(Symbol::EOS);
                }

                rule
            })
            .collect()
    }
}

/// A syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Syntax<'syntax, Def>(Cow<'syntax, [Rule<'syntax, Def>]>)
where
    Def: Clone;

impl<'syntax, Def> IterSymbolIdentifiers<'syntax> for Syntax<'syntax, Def>
where
    Def: IterSymbolIdentifiers<'syntax> + Clone,
{
    fn iter_symbol_identifiers(&self) -> impl Iterator<Item = SymbolId<'syntax>> {
        self.as_ref()
            .iter()
            .flat_map(IterSymbolIdentifiers::iter_symbol_identifiers)
            .dedup()
    }
}

impl<'syntax, Def, Ctx> TransformSyntax<'syntax, Ctx> for Syntax<'syntax, Def>
where
    Def: TransformSyntax<'syntax, Ctx> + Clone,
    Def::Transformed: Clone + 'syntax,
{
    type Transformed = Syntax<'syntax, Def::Transformed>;

    fn transform_syntax(self, ctx: &mut Ctx) -> Self::Transformed {
        self.into_iter()
            .map(|rule| rule.transform_syntax(ctx))
            .collect()
    }
}

impl<'syntax, Def> Syntax<'syntax, Def>
where
    Def: Clone,
{
    pub const fn from_borrow(rules: &'syntax [Rule<'syntax, Def>]) -> Self {
        Self(Cow::Borrowed(rules))
    }

    pub fn iter_rules_by_symbol_identifier<SymId: AsRef<str>>(
        &self,
        id: SymId,
    ) -> impl Iterator<Item = &Rule<'syntax, Def>> {
        self.as_ref().iter().filter(move |rule| rule.lhs.is(&id))
    }

    pub fn is_non_terminal<SymId: AsRef<str>>(&self, id: SymId) -> bool {
        self.iter_rules_by_symbol_identifier(id.as_ref())
            .any(|_| true)
    }

    pub fn is_terminal<SymId: AsRef<str>>(&self, id: SymId) -> bool {
        !self.is_non_terminal(id)
    }
}

impl<'syntax, Def> FromIterator<Rule<'syntax, Def>> for Syntax<'syntax, Def>
where
    Def: Clone,
{
    fn from_iter<T: IntoIterator<Item = Rule<'syntax, Def>>>(iter: T) -> Self {
        Self(Cow::Owned(iter.into_iter().collect()))
    }
}

impl<'syntax, Def> IntoIterator for Syntax<'syntax, Def>
where
    Def: Clone,
{
    type Item = Rule<'syntax, Def>;
    type IntoIter = CowIntoIter<'syntax, Rule<'syntax, Def>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'syntax, Def> AsRef<[Rule<'syntax, Def>]> for Syntax<'syntax, Def>
where
    Def: Clone,
{
    fn as_ref(&self) -> &[Rule<'syntax, Def>] {
        self.0.as_ref()
    }
}

impl<'syntax, Def> AsMut<Vec<Rule<'syntax, Def>>> for Syntax<'syntax, Def>
where
    Def: Clone,
{
    fn as_mut(&mut self) -> &mut Vec<Rule<'syntax, Def>> {
        self.0.to_mut()
    }
}

/// A production rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule<'syntax, Def> {
    pub lhs: SymbolId<'syntax>,
    pub rhs: Def,
}

impl<'syntax, Def> Rule<'syntax, Def> {
    pub const fn new(lhs: SymbolId<'syntax>, rhs: Def) -> Self {
        Self { lhs, rhs }
    }
}

impl<'syntax, Def> IterSymbolIdentifiers<'syntax> for Rule<'syntax, Def>
where
    Def: IterSymbolIdentifiers<'syntax>,
{
    fn iter_symbol_identifiers(&self) -> impl Iterator<Item = SymbolId<'syntax>> {
        std::iter::once(self.lhs.clone()).chain(self.rhs.iter_symbol_identifiers())
    }
}

impl<'syntax, Def, Ctx> TransformSyntax<'syntax, Ctx> for Rule<'syntax, Def>
where
    Def: TransformSyntax<'syntax, Ctx>,
{
    type Transformed = Rule<'syntax, Def::Transformed>;

    fn transform_syntax(self, ctx: &mut Ctx) -> Self::Transformed {
        Self::Transformed {
            lhs: self.lhs,
            rhs: self.rhs.transform_syntax(ctx),
        }
    }
}

/// A rule with only one definition.
pub type SingleRule<'syntax, Term> = Rule<'syntax, Definition<'syntax, Term>>;

/// A rule with multiple definitions.
pub type MultiRule<'syntax, Term> = Rule<'syntax, DefinitionSet<'syntax, Term>>;

impl<'syntax, Term> MultiRule<'syntax, Term>
where
    Term: Clone,
{
    /// Flatten a multi rule.
    pub fn flatten(self) -> impl Iterator<Item = SingleRule<'syntax, Term>> {
        let lhs = self.lhs;
        self.rhs.into_iter().map(move |rhs| SingleRule {
            lhs: lhs.clone(),
            rhs,
        })
    }
}

/// A set of definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionSet<'syntax, Term>(Cow<'syntax, [Definition<'syntax, Term>]>)
where
    Term: Clone;

impl<'syntax, Term> IterSymbolIdentifiers<'syntax> for DefinitionSet<'syntax, Term>
where
    Term: IterSymbolIdentifiers<'syntax> + Clone,
{
    fn iter_symbol_identifiers(&self) -> impl Iterator<Item = SymbolId<'syntax>> {
        self.as_ref()
            .iter()
            .flat_map(IterSymbolIdentifiers::iter_symbol_identifiers)
    }
}

impl<'syntax, Term, Ctx> TransformSyntax<'syntax, Ctx> for DefinitionSet<'syntax, Term>
where
    Term: TransformSyntax<'syntax, Ctx> + Clone,
    Term::Transformed: Clone + 'syntax,
{
    type Transformed = DefinitionSet<'syntax, Term::Transformed>;

    fn transform_syntax(self, ctx: &mut Ctx) -> Self::Transformed {
        self.into_iter()
            .map(|single| single.transform_syntax(ctx))
            .collect()
    }
}

impl<'syntax, Term> DefinitionSet<'syntax, Term>
where
    Term: Clone,
{
    pub const fn from_borrow(definitions: &'syntax [Definition<'syntax, Term>]) -> Self {
        Self(Cow::Borrowed(definitions))
    }
}

impl<'syntax, Term> IntoIterator for DefinitionSet<'syntax, Term>
where
    Term: Clone,
{
    type Item = Definition<'syntax, Term>;
    type IntoIter = CowIntoIter<'syntax, Definition<'syntax, Term>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'syntax, Term> AsRef<[Definition<'syntax, Term>]> for DefinitionSet<'syntax, Term>
where
    Term: Clone,
{
    fn as_ref(&self) -> &[Definition<'syntax, Term>] {
        self.0.as_ref()
    }
}

impl<'syntax, Term> AsMut<Vec<Definition<'syntax, Term>>> for DefinitionSet<'syntax, Term>
where
    Term: Clone,
{
    fn as_mut(&mut self) -> &mut Vec<Definition<'syntax, Term>> {
        self.0.to_mut()
    }
}

impl<'syntax, Term> FromIterator<Definition<'syntax, Term>> for DefinitionSet<'syntax, Term>
where
    Term: Clone,
{
    fn from_iter<T: IntoIterator<Item = Definition<'syntax, Term>>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

/// A simple definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Definition<'syntax, Term>(Cow<'syntax, [Term]>)
where
    Term: Clone;

impl<'syntax, Term> IterSymbolIdentifiers<'syntax> for Definition<'syntax, Term>
where
    Term: IterSymbolIdentifiers<'syntax> + Clone,
{
    fn iter_symbol_identifiers(&self) -> impl Iterator<Item = SymbolId<'syntax>> {
        self.as_ref()
            .iter()
            .flat_map(IterSymbolIdentifiers::iter_symbol_identifiers)
    }
}

impl<'syntax, Term, Ctx> TransformSyntax<'syntax, Ctx> for Definition<'syntax, Term>
where
    Term: TransformSyntax<'syntax, Ctx> + Clone,
    Term::Transformed: Clone + 'syntax,
{
    type Transformed = Definition<'syntax, Term::Transformed>;

    fn transform_syntax(self, ctx: &mut Ctx) -> Self::Transformed {
        self.into_iter()
            .map(|term| term.transform_syntax(ctx))
            .collect()
    }
}

impl<'syntax, Term> Definition<'syntax, Term>
where
    Term: Clone,
{
    pub const fn from_borrow(terms: &'syntax [Term]) -> Self {
        Self(Cow::Borrowed(terms))
    }
}

impl<'syntax, Term> IntoIterator for Definition<'syntax, Term>
where
    Term: Clone,
{
    type Item = Term;
    type IntoIter = CowIntoIter<'syntax, Term>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'syntax, Term> FromIterator<Term> for Definition<'syntax, Term>
where
    Term: Clone,
{
    fn from_iter<T: IntoIterator<Item = Term>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<'syntax, Term> AsRef<[Term]> for Definition<'syntax, Term>
where
    Term: Clone,
{
    fn as_ref(&self) -> &[Term] {
        self.0.as_ref()
    }
}

impl<'syntax, Term> AsMut<Vec<Term>> for Definition<'syntax, Term>
where
    Term: Clone,
{
    fn as_mut(&mut self) -> &mut Vec<Term> {
        self.0.to_mut()
    }
}
