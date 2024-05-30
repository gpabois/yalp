use std::{
    collections::HashSet,
    ops::{BitAnd, BitOr, Not, Range, RangeInclusive},
};

use crate::dfa;

/// A set of chars
pub enum CharSet {
    And(And),
    Or(Or),
    Gt(Gt),
    Gte(Gte),
    Lt(Lt),
    Lte(Lte),
    Eq(Eq),
    NotEq(NotEq),
    In(In),
    NotIn(NotIn),
    All,
    Epsilon,
}

impl CharSet {
    pub fn eq(ch: char) -> Self {
        Self::Eq(Eq(ch))
    }

    pub fn gt(ch: char) -> Self {
        Self::Gt(Gt(ch))
    }
    pub fn gte(ch: char) -> Self {
        Self::Gte(Gte(ch))
    }

    pub fn lt(ch: char) -> Self {
        Self::Lt(Lt(ch))
    }

    pub fn lte(ch: char) -> Self {
        Self::Lte(Lte(ch))
    }

    pub fn r#in<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = char>,
    {
        Self::In(In(iter.into_iter().collect()))
    }
}

impl dfa::Set for CharSet {
    type Item = char;

    fn intersect(lhs: Self, rhs: Self) -> Self {
        lhs & rhs
    }

    fn union(lhs: Self, rhs: Self) -> Self {
        lhs | rhs
    }

    fn difference(lhs: Self, rhs: Self) -> Self {
        lhs & !rhs
    }

    fn is_empty(&self) -> bool {
        match self {
            CharSet::And(a) => a.is_empty(),
            CharSet::Or(a) => a.is_empty(),
            CharSet::Gt(a) => a.is_empty(),
            CharSet::Gte(a) => a.is_empty(),
            CharSet::Lt(a) => a.is_empty(),
            CharSet::Lte(a) => a.is_empty(),
            CharSet::Eq(a) => a.is_empty(),
            CharSet::NotEq(a) => a.is_empty(),
            CharSet::In(a) => a.is_empty(),
            CharSet::NotIn(a) => a.is_empty(),
            CharSet::All => false,
            CharSet::Epsilon => true,
        }
    }

    fn contains(&self, ch: &Self::Item) -> bool {
        match self {
            CharSet::And(a) => a.contains(ch),
            CharSet::Or(a) => a.contains(ch),
            CharSet::Gt(a) => a.contains(ch),
            CharSet::Gte(a) => a.contains(ch),
            CharSet::Lt(a) => a.contains(ch),
            CharSet::Lte(a) => a.contains(ch),
            CharSet::Eq(a) => a.contains(ch),
            CharSet::NotEq(a) => a.contains(ch),
            CharSet::In(a) => a.contains(ch),
            CharSet::NotIn(a) => a.contains(ch),
            CharSet::All => true,
            CharSet::Epsilon => false,
        }
    }
}

impl From<Range<char>> for CharSet {
    fn from(value: Range<char>) -> Self {
        CharSet::gte(value.start) & CharSet::lt(value.end)
    }
}

impl From<RangeInclusive<char>> for CharSet {
    fn from(value: RangeInclusive<char>) -> Self {
        CharSet::gte(*value.start()) & CharSet::lte(*value.end())
    }
}

impl From<char> for CharSet {
    fn from(value: char) -> Self {
        CharSet::Eq(Eq(value))
    }
}

impl From<And> for CharSet {
    fn from(value: And) -> Self {
        if value.is_empty() {
            Self::Epsilon
        } else {
            Self::And(value)
        }
    }
}

impl From<Or> for CharSet {
    fn from(value: Or) -> Self {
        if value.is_empty() {
            Self::Epsilon
        } else {
            Self::Or(value)
        }
    }
}

impl From<Gt> for CharSet {
    fn from(value: Gt) -> Self {
        if value.is_empty() {
            Self::Epsilon
        } else {
            Self::Gt(value)
        }
    }
}

impl From<Gte> for CharSet {
    fn from(value: Gte) -> Self {
        Self::Gte(value)
    }
}

impl From<Lt> for CharSet {
    fn from(value: Lt) -> Self {
        if value.is_empty() {
            Self::Epsilon
        } else {
            Self::Lt(value)
        }
    }
}

impl From<Lte> for CharSet {
    fn from(value: Lte) -> Self {
        CharSet::Lte(value)
    }
}

impl From<Eq> for CharSet {
    fn from(value: Eq) -> Self {
        CharSet::Eq(value)
    }
}

impl From<NotEq> for CharSet {
    fn from(value: NotEq) -> Self {
        CharSet::NotEq(value)
    }
}

impl From<In> for CharSet {
    fn from(value: In) -> Self {
        if value.is_empty() {
            Self::Epsilon
        } else {
            CharSet::In(value)
        }
    }
}

impl From<NotIn> for CharSet {
    fn from(value: NotIn) -> Self {
        if value.is_empty() {
            Self::Epsilon
        } else {
            CharSet::NotIn(value)
        }
    }
}

impl CharSet {
    pub fn contains(&self, ch: &char) -> bool {
        match self {
            CharSet::And(a) => a.contains(ch),
            CharSet::Or(a) => a.contains(ch),
            CharSet::Gt(a) => a.contains(ch),
            CharSet::Gte(a) => a.contains(ch),
            CharSet::Lt(a) => a.contains(ch),
            CharSet::Lte(a) => a.contains(ch),
            CharSet::Eq(a) => a.contains(ch),
            CharSet::NotEq(a) => a.contains(ch),
            CharSet::In(a) => a.contains(ch),
            CharSet::NotIn(a) => a.contains(ch),
            CharSet::All => true,
            CharSet::Epsilon => false,
        }
    }
}

impl BitOr for CharSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Or(vec![self, rhs]).into()
    }
}

impl BitAnd for CharSet {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        And(vec![self, rhs]).into()
    }
}

impl Not for CharSet {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            CharSet::And(a) => a.not().into(),
            CharSet::Or(a) => a.not().into(),
            CharSet::Gt(a) => a.not().into(),
            CharSet::Gte(a) => a.not().into(),
            CharSet::Lt(a) => a.not().into(),
            CharSet::Lte(a) => a.not().into(),
            CharSet::Eq(a) => a.not().into(),
            CharSet::NotEq(a) => a.not().into(),
            CharSet::In(a) => a.not().into(),
            CharSet::NotIn(a) => a.not().into(),
            CharSet::All => CharSet::Epsilon,
            CharSet::Epsilon => CharSet::All,
        }
    }
}

pub struct In(HashSet<char>);
impl In {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains(&self, ch: &char) -> bool {
        self.0.contains(ch)
    }
}
impl Not for In {
    type Output = NotIn;

    fn not(self) -> Self::Output {
        NotIn(self.0)
    }
}

pub struct NotIn(HashSet<char>);
impl Not for NotIn {
    type Output = In;

    fn not(self) -> Self::Output {
        In(self.0)
    }
}
impl NotIn {
    pub fn contains(&self, ch: &char) -> bool {
        !self.0.contains(ch)
    }
    pub fn is_empty(&self) -> bool {
        self.0.len() == (char::MAX as usize) + 1
    }
}

pub struct And(Vec<CharSet>);
impl And {
    pub fn contains(&self, ch: &char) -> bool {
        self.0.iter().all(|a| a.contains(ch))
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().any(dfa::Set::is_empty)
    }
}

impl Not for And {
    type Output = Or;

    fn not(self) -> Self::Output {
        Or(self.0.into_iter().map(CharSet::not).collect())
    }
}
pub struct Or(Vec<CharSet>);
impl Or {
    pub fn contains(&self, ch: &char) -> bool {
        self.0.iter().any(|a| a.contains(ch))
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(dfa::Set::is_empty)
    }
}
impl Not for Or {
    type Output = And;

    fn not(self) -> Self::Output {
        And(self.0.into_iter().map(CharSet::not).collect())
    }
}

pub struct Gt(char);
impl Gt {
    pub fn contains(&self, ch: &char) -> bool {
        *ch > self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0 == char::MAX
    }
}
impl Not for Gt {
    type Output = Lte;

    fn not(self) -> Self::Output {
        Lte(self.0)
    }
}

pub struct Gte(char);
impl Gte {
    pub fn contains(&self, ch: &char) -> bool {
        *ch >= self.0
    }

    pub fn is_empty(&self) -> bool {
        true
    }
}
impl Not for Gte {
    type Output = Lt;

    fn not(self) -> Self::Output {
        Lt(self.0)
    }
}

pub struct Lt(char);
impl Lt {
    pub fn contains(&self, ch: &char) -> bool {
        *ch < self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0 == '\0'
    }
}
impl Not for Lt {
    type Output = Gte;

    fn not(self) -> Gte {
        Gte(self.0)
    }
}

pub struct Lte(char);
impl Lte {
    pub fn contains(&self, ch: &char) -> bool {
        *ch <= self.0
    }

    pub fn is_empty(&self) -> bool {
        true
    }
}
impl Not for Lte {
    type Output = Gt;

    fn not(self) -> Self::Output {
        Gt(self.0)
    }
}
pub struct Eq(char);
impl Eq {
    pub fn contains(&self, ch: &char) -> bool {
        *ch == self.0
    }

    pub fn is_empty(&self) -> bool {
        false
    }
}
impl Not for Eq {
    type Output = NotEq;

    fn not(self) -> Self::Output {
        NotEq(self.0)
    }
}

pub struct NotEq(char);
impl NotEq {
    pub fn contains(&self, ch: &char) -> bool {
        *ch != self.0
    }

    pub fn is_empty(&self) -> bool {
        false
    }
}
impl Not for NotEq {
    type Output = Eq;

    fn not(self) -> Self::Output {
        Eq(self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::dfa::Set as _;

    use super::CharSet;

    #[test]
    fn test_eq() {
        let at = CharSet::eq('a');
        assert!(at.contains(&'a'));
        assert!(!at.contains(&'b'));
    }

    #[test]
    fn test_gte() {
        let at = CharSet::gte('c');

        assert!(!at.contains(&'a'));
        assert!(at.contains(&'c'));
        assert!(at.contains(&'d'));
    }

    #[test]
    fn test_gt() {
        let at = CharSet::gt('c');

        assert!(!at.contains(&'a'));
        assert!(!at.contains(&'c'));
        assert!(at.contains(&'d'));
    }

    #[test]
    fn test_lte() {
        let at = CharSet::lte('c');
        assert!(at.contains(&'a'));
        assert!(at.contains(&'c'));
        assert!(!at.contains(&'d'));
    }

    #[test]
    fn test_lt() {
        let at = CharSet::lt('c');
        assert!(at.contains(&'a'));
        assert!(!at.contains(&'c'));
        assert!(!at.contains(&'d'));
    }

    #[test]
    fn test_in() {
        let at = CharSet::r#in(['a', '0', '#']);

        assert!(at.contains(&'a'));
        assert!(at.contains(&'0'));
        assert!(at.contains(&'#'));
        assert!(!at.contains(&'b'));
    }

    #[test]
    fn test_range() {
        let at = CharSet::from('a'..='e');

        assert!(at.contains(&'a'));
        assert!(at.contains(&'b'));
        assert!(!at.contains(&'f'));
    }

    #[test]
    fn test_intersect() {
        let left = CharSet::from('a'..='z');
        let right = CharSet::from('b'..='f');

        let int = CharSet::intersect(left, right);
        assert!(!int.contains(&'a'));
        assert!(int.contains(&'e'));
    }

    #[test]
    fn test_difference() {
        let left = CharSet::from('a'..='z');
        let right = CharSet::from('b'..='f');

        let at = CharSet::difference(left, right);
        assert!(at.contains(&'a'));
        assert!(!at.contains(&'b'));
        assert!(!at.contains(&'f'));
        assert!(at.contains(&'z'));
    }
}
