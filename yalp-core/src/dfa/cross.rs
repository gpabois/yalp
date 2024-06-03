use std::{collections::HashMap, ops::Deref};

use itertools::Itertools;

use super::{
    graph::{ActionSequence, Edge, EdgeSet, Graph, IntoGraph, Node},
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

        let mut stack = vec![CrossNode::Start];
        let mut visited = Vec::<CrossNode>::default();

        while let Some(cross_node) = stack.pop() {
            if visited.contains(&cross_node) {
                continue;
            }

            visited.push(cross_node);
            
            match cross_node {
                CrossNode::Start => {
                    let ledges: EdgeSet<_, _> = cross_graph.left.iter_entering_edges().cloned().collect();
                    let redges: EdgeSet<_, _> = cross_graph.right.iter_entering_edges().cloned().collect();
                    let cedges = ledges * redges;
                    
                    cedges.iter().for_each(|edge| {
                        stack.push(edge.to)
                    });

                    cross_graph.edges.extend(cedges);             
                },
                CrossNode::Left(lhs) => {
                    for edge in cross_graph.left.iter_follow(lhs) {
                        cross_graph.edges.push(CrossEdge {
                            from: CrossNode::left(lhs),
                            to: CrossNode::left(edge.to),
                            priority: edge.priority,
                            set: edge.set.clone(),
                            actions: edge.actions.clone(),
                        });
                        stack.push(CrossNode::left(edge.to));
                    }
                },
                CrossNode::Right(rhs) => {
                    for edge in cross_graph.left.iter_follow(rhs) {
                        cross_graph.edges.push(CrossEdge {
                            from: CrossNode::right(rhs),
                            to: CrossNode::right(edge.to),
                            priority: edge.priority,
                            set: edge.set.clone(),
                            actions: edge.actions.clone(),
                        });
                        stack.push(CrossNode::right(edge.to));
                    }
                },
                CrossNode::Shared(lhs, rhs) => {
                    let ledges: EdgeSet<_, _> = cross_graph.left.iter_follow(lhs).cloned().collect();
                    let redges: EdgeSet<_, _> = cross_graph.right.iter_follow(rhs).cloned().collect();
                    let cedges = ledges * redges;
                    
                    cedges.iter().for_each(|edge| {
                        stack.push(edge.to)
                    });

                    cross_graph.edges.extend(cedges);
                },
                CrossNode::End => {}
            }
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
            from: CrossNode::left(self.from),
            to: CrossNode::left(self.to),
            priority: self.priority,
            // E1 - E2
            set: S::difference(self.set.clone(), rhs.set.clone()),
            actions: self.actions.clone(),

        };

        let right = CrossEdge {
            from: CrossNode::right(rhs.from),
            to: CrossNode::right(rhs.to),
            priority: self.priority,
            // E2 - E1
            set: S::difference(rhs.set.clone(), self.set.clone()),
            actions: rhs.actions.clone(),
        };

        let shared = CrossEdge {
            from: CrossNode::shared(self.from, rhs.from),
            to: CrossNode::shared(self.to, rhs.to),
            priority: self.priority,
            // E1 ^ E2
            set: S::intersect(rhs.set.clone(), self.set.clone()),
            actions: rhs.actions + self.actions,
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

impl<S, A> IntoGraph<S,A> for CrossGraph<S,A> 
where S: Set
{
    fn into_graph(self) -> Graph<S, A> {
        let mut table = HashMap::<CrossNode, Node>::default();
        let mut graph = Graph::<S,A>::default();

        for edge in self.edges.into_iter() {
            let from = if edge.from.is_start() { 
                Node::Start 
                } else if edge.from.is_end() { 
                    Node::End 
                } else { 
                    if let Some(node) = table.get(&edge.from).copied() {
                        node
                    } else {
                        let node = graph.add();
                        table.insert(edge.from, node);
                        node
                    }
                };

             let to = if edge.to.is_start() { 
                    Node::Start 
                } else if edge.to.is_end() { 
                    Node::End 
                } else { 
                    if let Some(node) = table.get(&edge.to).copied() {
                        node
                    } else {
                        let node = graph.add();
                        table.insert(edge.to, node);
                        node
                    }
                };

            graph.edges.push(Edge {
                from,
                to,
                priority: edge.priority,
                set: edge.set,
                actions: edge.actions
            });
        }

        graph
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum CrossNode {
    Start,
    Left(Node),
    Right(Node),
    Shared(Node, Node),
    End
}

impl CrossNode {
    pub fn is_start(&self) -> bool {
        matches!(self, CrossNode::Start)
    }
    
    pub fn is_end(&self) -> bool {
        matches!(self, CrossNode::End)
    }

    pub fn left(lhs: Node) -> Self {
        if lhs.is_end() {
            return Self::End
        }

        Self::Left(lhs)
    }

    pub fn right(rhs: Node) -> Self {
        if rhs.is_end() {
            return Self::End
        }

        Self::Right(rhs)
    }

    pub fn shared(lhs: Node, rhs: Node) -> Self {
        if lhs.is_end() && rhs.is_end() {
            return Self::End
        }

        Self::Shared(lhs, rhs)
    }
}

impl std::hash::Hash for CrossNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

#[derive(Clone)]
pub struct CrossEdge<S, A> {
    from: CrossNode,
    to: CrossNode,
    priority: isize,
    set: S,
    actions: ActionSequence<A>,
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
            priority: lhs.priority,
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
    pub fn push(&mut self, edge: CrossEdge<S,A>) {
        self.0.push(edge)
    }
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
}
