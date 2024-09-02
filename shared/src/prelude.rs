use std::borrow::Cow;

use crate::symbol::SymbolId;

/// Transform a syntax node into a symbol id.
pub trait IntoSymbolIdentifier<'syntax> {
    fn into_symbol_identifier(self) -> SymbolId<'syntax>;
}

pub trait IterSymbolIdentifiers<'syntax> {
    fn iter_symbol_identifiers(&self) -> impl Iterator<Item = SymbolId<'syntax>>;
}

/// Process a syntax declaration.
///
/// It transforms a symbol id into a symbol with contextual informations
/// (is it an EOS, a terminal, a non terminal, etc...)
pub trait TransformSyntax<'syntax, Ctx> {
    type Transformed;

    fn transform_syntax(self, ctx: &mut Ctx) -> Self::Transformed;
}

pub enum CowIntoIter<'a, T: 'a> {
    Borrowed(std::iter::Cloned<std::slice::Iter<'a, T>>),
    Owned(<Vec<T> as IntoIterator>::IntoIter),
}

impl<'a, T: Clone + 'a> Iterator for CowIntoIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CowIntoIter::Borrowed(iter) => iter.next(),
            CowIntoIter::Owned(iter) => iter.next(),
        }
    }
}

pub trait CowIntoIterator<'a> {
    type Item;
    type IntoIter: Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter;
}

impl<'a, T: Clone + 'a> CowIntoIterator<'a> for Cow<'a, [T]> {
    type Item = T;

    type IntoIter = CowIntoIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Cow::Borrowed(slice) => CowIntoIter::Borrowed(slice.iter().cloned()),
            Cow::Owned(vec) => CowIntoIter::Owned(vec.into_iter()),
        }
    }
}
