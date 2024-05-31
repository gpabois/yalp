use std::{collections::HashSet, default, ops::Deref};

use itertools::Itertools;

use super::{
    graph::{ActionSequence, Edge, EdgeSet, Graph, Node},
    Set,
};

impl<S, A> std::ops::Mul<Graph<S, A>> for Graph<S, A>
where
    S: Set + Clone,
    A: Clone,
{
    type Output = CrossGraph<S, A>;

    fn mul(self, rhs: Graph<S, A>) -> Self::Output {
        let mut cross_graph = CrossGraph::new(self, rhs);

        let mut stack = vec![(
            cross_graph.left.iter_entering_edges()
                .cloned()
                .collect::<EdgeSet<S, A>>(),
            cross_graph.right.iter_entering_edges()
                .cloned()
                .collect::<EdgeSet<S, A>>(),
        )];

        let mut lhs_visited = HashSet::<Node>::default();
        let mut rhs_visited = HashSet::<Node>::default();

        while let Some((lhs_edges, rhs_edges)) = stack.pop() {
            let cross = lhs_edges * rhs_edges; 
            cross_graph.edges.extend(cross.clone());

            let (lhs_nodes, rhs_nodes) = cross.get_following_nodes();
            
            let lhs_edges: EdgeSet<S,A> = lhs_nodes.into_iter().flat_map(|from| {
                cross_graph.left.iter_follow(from)
            }).cloned().collect();
            
            let rhs_edges: EdgeSet<S,A>  = rhs_nodes.into_iter().flat_map(|from| {
                cross_graph.right.iter_follow(from)
            }).cloned().collect();

            stack.push((lhs_edges, rhs_edges));
        }

        cross_graph
    }
}

impl<S, A> std::ops::Mul<Self> for Edge<S, A>
where
    S: Set + Clone,
    A: Clone,
{
    type Output = Vec<CrossEdge<S, A>>;

    fn mul(self, rhs: Self) -> Vec<CrossEdge<S, A>> {
        let left = CrossEdge {
            from: CrossNode::Left(self.from),
            // E1 - E2
            set: S::difference(self.set.clone(), rhs.set.clone()),
            actions: self.actions.clone(),
            to: CrossNode::Left(self.to),
        };

        let right = CrossEdge {
            from: CrossNode::Right(rhs.from),
            // E2 - E1
            set: S::difference(rhs.set.clone(), self.set.clone()),
            actions: rhs.actions.clone(),
            to: CrossNode::Right(rhs.to),
        };

        let shared = CrossEdge {
            from: CrossNode::Shared(self.from, rhs.from),
            // E1 ^ E2
            set: S::intersect(rhs.set.clone(), self.set.clone()),
            actions: rhs.actions + self.actions,
            to: CrossNode::Shared(self.to, rhs.to),
        };

        [left, right, shared]
            .into_iter()
            .filter(|e| !e.is_empty())
            .collect()
    }
}

impl<S, A> std::ops::Mul<Self> for EdgeSet<S, A>
where
    S: Set + Clone,
    A: Clone,
{
    type Output = CrossEdgeSet<S, A>;

    fn mul(self, rhs: Self) -> Self::Output {
        let lhs = self;

        lhs.cartesian_product(rhs)
            .flat_map(|(lhs, rhs)| lhs * rhs)
            .collect::<CrossEdgeSet<S, A>>()
            .merge()
    }
}

pub struct CrossGraph<S, A> {
    pub left: Graph<S, A>,
    pub right: Graph<S, A>,
    pub edges: CrossEdgeSet<S, A>,
}

impl<S,A> CrossGraph<S,A> {
    pub fn new(left: Graph<S,A>, right: Graph<S,A>) -> Self {
        Self {
            left,
            right,
            edges: CrossEdgeSet::default()
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum CrossNode {
    Left(Node),
    Right(Node),
    Shared(Node, Node),
}

impl std::hash::Hash for CrossNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

#[derive(Clone)]
pub struct CrossEdge<S, A> {
    from: CrossNode,
    set: S,
    actions: ActionSequence<A>,
    to: CrossNode,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct CrossEdgeId {
    from: CrossNode,
    to: CrossNode,
}

impl<S, A> CrossEdge<S, A> {
    pub fn id(&self) -> CrossEdgeId {
        CrossEdgeId {
            from: self.from,
            to: self.to,
        }
    }

    /// Two cross edges are similar if they share the same source and destination.
    pub fn are_similar(&self, rhs: &Self) -> bool {
        self.id() == rhs.id()
    }
}
impl<C, A> CrossEdge<C, A>
where
    C: Set,
{
    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    /// Merge two similar edges
    pub fn merge(lhs: Self, rhs: Self) -> Self {
        if lhs.are_similar(&rhs) {
            panic!("not mergeable");
        }

        Self {
            from: lhs.from,
            to: lhs.to,
            actions: lhs.actions + rhs.actions,
            set: C::union(lhs.set, rhs.set),
        }
    }
}

/// A set of cross edges
#[derive(Clone)]
pub struct CrossEdgeSet<S, A>(Vec<CrossEdge<S, A>>);

impl<S, A> Default for CrossEdgeSet<S, A> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<S,A> IntoIterator for CrossEdgeSet<S,A> {
    type Item = CrossEdge<S,A>;

    type IntoIter = <Vec<CrossEdge<S, A>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<S, A> FromIterator<CrossEdge<S, A>> for CrossEdgeSet<S, A> {
    fn from_iter<T: IntoIterator<Item = CrossEdge<S, A>>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<S, A> Deref for CrossEdgeSet<S, A> {
    type Target = [CrossEdge<S, A>];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S,A> CrossEdgeSet<S,A> {
    pub fn extend<I: IntoIterator<Item=CrossEdge<S,A>>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<S, A> CrossEdgeSet<S, A>
where
    S: Set,
{
    /// Merge similar edges
    pub fn merge(self) -> CrossEdgeSet<S, A> {
        self.0
            .into_iter()
            .group_by(CrossEdge::id)
            .into_iter()
            .flat_map(|(_, g)| g.into_iter().reduce(CrossEdge::merge))
            .collect()
    }

    pub fn get_following_nodes(self) -> (HashSet<Node>, HashSet<Node>) {
        let mut lhs = HashSet::<Node>::default();
        let mut rhs = HashSet::<Node>::default();

        for edge in self.iter() {
            match edge.to {
                CrossNode::Left(l) => {
                    lhs.insert(l);
                },
                CrossNode::Right(r) => {
                    rhs.insert(r);
                },
                CrossNode::Shared(l, r) => {
                    lhs.insert(l);
                    rhs.insert(r);
                }
            }
        }

        (lhs, rhs)
    }
}
