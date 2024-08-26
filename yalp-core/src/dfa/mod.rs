/*!
  Deterministic finite automaton (DFA)
*/

use std::collections::HashMap;

pub use graph::{ActionSequence, Edge, Graph, Node};
use itertools::Itertools;

pub mod cross;
pub mod graph;

pub use graph::IntoGraph;
/// A trait defining set-related basic operations.
pub trait Set {
    type Item;

    fn intersect(lhs: Self, rhs: Self) -> Self;
    fn union(lhs: Self, rhs: Self) -> Self;
    fn difference(lhs: Self, rhs: Self) -> Self;
    fn is_empty(&self) -> bool;
    fn contains(&self, item: &Self::Item) -> bool;
}

struct Transition<S,A> {
  set: S,
  priority: isize,
  actions: ActionSequence<A>,
  to: State
}

impl<S,A> From<Edge<S,A>> for Transition<S,A> {
    fn from(value: Edge<S,A>) -> Self {
        Self {
          set: value.set,
          priority: value.priority,
          actions: value.actions,
          to: value.to.into()
        }
    }
}
struct Row<S,A>(Vec<Transition<S,A>>) where S: Set;

impl<S,A> Row<S,A> where S: Set {
  pub fn find_transition(&self, item: &S::Item) -> Option<&Transition<S,A>> {
    self.0
      .iter()
      .filter(|trs| trs.set.contains(item))
      .sorted_by(|a,b| a.priority.cmp(&b.priority))
      .next()
  }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum State {
  Start,
  Internal(usize),
  End
}

impl std::hash::Hash for State {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl From<Node> for State {
    fn from(value: Node) -> Self {
        match value {
            Node::Start => State::Start,
            Node::Internal(i) => State::Internal(i),
            Node::End => State::End,
        }
    }
}

/// The DFA state table.
pub struct Table<S,A>(HashMap<State, Row<S,A>>) where S: Set;

impl<S,A> Table<S,A> where S: Set + Clone, A: Clone{
  pub fn new<G: IntoGraph<S,A>>(value: G) -> Self {
    let graph = value.into_graph();
    Self::from(graph)
  }
}

impl<S,A> Table<S,A> where S: Set {
  /// Find the next state
  pub fn next_state(&self, from: &State, item: &S::Item) -> Option<(State, &ActionSequence<A>)> {
    self.0.get(from)
      .and_then(|row| row.find_transition(item))
      .map(|trs| (trs.to, &trs.actions))
  }
}

impl<S,A> From<Graph<S,A>> for Table<S,A> where S: Set + Clone, A: Clone {
    fn from(value: Graph<S,A>) -> Self {
        Self(
          value.edges
          .into_iter()
          .group_by(|e| e.from)
          .into_iter()
          .map(|(from, edges)| {
            let transitions: Vec<_> = edges.into_iter().cloned().map(Transition::from).collect();
            (State::from(from), Row(transitions))
          }).collect()
        )
    }
}