use std::ops::{Deref, DerefMut};

use itertools::Itertools;

use super::Set;

pub trait IntoGraph<S, A>
where
    S: Set,
{
    fn into_graph(self) -> Graph<S, A>;
}

#[derive(Default)]
pub struct Graph<S, A> {
    edges: EdgeSet<S, A>,
    offset: usize,
    count: usize,
}

impl<S, A> Graph<S, A> {
    /// Add a new node.
    pub fn add(&mut self) -> Node {
        let node = self.count;
        self.count += 1;
        node.into()
    }

    /// Connect two states
    pub fn on<I>(&mut self, from: Node, to: Node, set: S, actions: I)
    where
        I: IntoIterator<Item = A>,
    {
        self.edges.push(Edge {
            from,
            to,
            set,
            actions: actions.into_iter().collect(),
        })
    }

    pub fn iter_follow(&self, from: Node) -> impl Iterator<Item = &Edge<S, A>> {
        self.edges.iter().filter(move |edge| edge.from == from)
    }

    pub fn iter_entering_edges(&self) -> impl Iterator<Item = &Edge<S, A>> {
        self.edges.iter().filter(|edge| edge.from.is_start())
    }

    pub fn iter_leaving_edges(&self) -> impl Iterator<Item = &Edge<S, A>> {
        self.edges.iter().filter(|edge| edge.to.is_end())
    }

    pub fn iter_internal_edges(&self) -> impl Iterator<Item = &Edge<S, A>> {
        self.edges
            .iter()
            .filter(|edge| edge.to.is_internal() && edge.from.is_internal())
    }

    pub fn offset(&mut self, n: usize) {
        self.offset += n;
        self.edges.iter_mut().for_each(|edge| *edge += n);
    }
}

impl<S, A> Graph<S, A>
where
    S: Clone,
    A: Clone,
{
    /// Append a graph
    pub fn append(&mut self, mut rhs: Self) {
        rhs += self.offset + self.count;

        self.count += rhs.count;
        let mut edges = EdgeSet::default();

        edges.extend(self.iter_entering_edges().cloned());
        edges.extend(self.iter_internal_edges().cloned());
        edges.extend(rhs.iter_internal_edges().cloned());
        edges.extend(rhs.iter_leaving_edges().cloned());

        // connect lhs::leaving to rhs::entering
        edges.extend(self.iter_leaving_edges().flat_map(|leaving| {
            rhs.iter_entering_edges().map(|entering| Edge {
                from: leaving.from,
                set: entering.set.clone(),
                actions: entering.actions.clone() + leaving.actions.clone(),
                to: entering.to,
            })
        }));

        self.edges = edges;
    }
}

/// Offset the fragment
impl<S, A> std::ops::AddAssign<usize> for Graph<S, A> {
    fn add_assign(&mut self, rhs: usize) {
        self.offset(rhs)
    }
}

/// Append two fragments
impl<S, A> std::ops::Add<Self> for Graph<S, A>
where
    S: Clone,
    A: Clone,
{
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.append(rhs);
        self
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Node {
    Start,
    Internal(usize),
    End,
}

impl std::hash::Hash for Node {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl Node {
    pub fn is_start(&self) -> bool {
        matches!(self, Node::Start)
    }

    pub fn is_internal(&self) -> bool {
        matches!(self, Node::Internal(_))
    }

    pub fn is_end(&self) -> bool {
        matches!(self, Node::End)
    }
}

impl From<usize> for Node {
    fn from(value: usize) -> Self {
        Self::Internal(value)
    }
}

impl std::ops::AddAssign<usize> for Node {
    fn add_assign(&mut self, rhs: usize) {
        if let Node::Internal(value) = self {
            *value += rhs
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EdgeId {
    pub from: Node,
    pub to: Node,
}

pub struct Edge<S, A> {
    pub from: Node,
    /// If the set contains the item, then it is a valid state transition.
    pub set: S,
    /// The action to perform if the edge is taken.
    pub actions: ActionSequence<A>,
    pub to: Node,
}

impl<S, A> Edge<S, A> {
    pub fn id(&self) -> EdgeId {
        EdgeId {
            from: self.from,
            to: self.to,
        }
    }
}

impl<S, A> Clone for Edge<S, A>
where
    S: Clone,
    A: Clone,
{
    fn clone(&self) -> Self {
        Self {
            from: self.from,
            to: self.to,
            set: self.set.clone(),
            actions: self.actions.clone(),
        }
    }
}

impl<S, A> std::ops::AddAssign<usize> for Edge<S, A> {
    fn add_assign(&mut self, rhs: usize) {
        self.to += rhs;
        self.from += rhs;
    }
}

pub struct EdgeSet<S, A>(Vec<Edge<S, A>>);

impl<S, A> Default for EdgeSet<S, A> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<S, A> EdgeSet<S, A>
where
    S: Clone,
    A: Clone,
{
    pub fn cartesian_product(self, rhs: Self) -> impl Iterator<Item = (Edge<S, A>, Edge<S, A>)> {
        self.0.into_iter().cartesian_product(rhs.0.into_iter())
    }
}

impl<S, A> EdgeSet<S, A> {
    pub fn extend(&mut self, edges: impl IntoIterator<Item = Edge<S, A>>) {
        self.0.extend(edges);
    }

    pub fn push(&mut self, edge: Edge<S, A>) {
        self.0.push(edge)
    }
}

impl<S, A> Deref for EdgeSet<S, A> {
    type Target = [Edge<S, A>];
    fn deref(&self) -> &[Edge<S, A>] {
        &self.0
    }
}

impl<S, A> DerefMut for EdgeSet<S, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<S, A> FromIterator<Edge<S, A>> for EdgeSet<S, A> {
    fn from_iter<T: IntoIterator<Item = Edge<S, A>>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[derive(Default)]
pub struct ActionSequence<A>(Vec<A>);

impl<A> Clone for ActionSequence<A>
where
    A: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<A> std::ops::Add<Self> for ActionSequence<A> {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.0.extend(rhs.0);
        self
    }
}
impl<A> FromIterator<A> for ActionSequence<A> {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}
