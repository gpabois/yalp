/*!
  Deterministic finite automaton (DFA)
*/

pub mod charset;
pub mod cross;
pub mod graph;

/// A trait defining set-related basic operations.
pub trait Set {
    type Item;

    fn intersect(lhs: Self, rhs: Self) -> Self;
    fn union(lhs: Self, rhs: Self) -> Self;
    fn difference(lhs: Self, rhs: Self) -> Self;
    fn is_empty(&self) -> bool;
    fn contains(&self, item: &Self::Item) -> bool;
}
