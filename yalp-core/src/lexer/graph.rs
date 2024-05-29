use super::atomic::Atomic;

pub trait IntoGraph {
    fn into_graph(self) -> Graph;
}

pub trait Set {
    type Item;

    fn intersect(lhs: Self, rhs: Self) -> Self;
    fn union(lhs: Self, rhs: Self) -> Self;
    fn difference(lhs: Self, rhs: Self) -> Self;
    fn is_empty(&self) -> Self;
    fn contains(&self, item: &Self::Item) -> bool;
}

#[derive(Default)]
pub struct Graph {
    edges: Vec<Edge>,
    offset: usize,
    count: usize,
}

impl Graph<A> where A: Clone {
    pub fn add(&mut self) -> Node {
        let node = self.count;
        self.count += 1;
        node.into()
    }

    pub fn iter_entering_edges(&self) -> impl Iterator<Item=&Edge<A>> {
        self.edges.iter().filter(|edge| edge.from.is_start())
    }

    pub fn iter_leaving_edges(&self) -> impl Iterator<Item=&Edge<A>> {
        self.edges.iter().filter(|edge| edge.to.is_end())
    }

    pub fn iter_internal_edges(&self) -> impl Iterator<Item=&Edge<A>> {
        self.edges.iter().filter(|edge| edge.to.is_internal() && edge.from.is_internal())
    }

    pub fn offset(&mut self, n: usize) {
        self.offset += n;
        self.edges.iter_mut().for_each(|edge| *edge += n);
    }

    /// Append two graphes
    pub fn append(&mut self, mut rhs: Self) {
        rhs += self.offset + self.count;

        self.count += rhs.count;
        let mut edges: Vec<Edge<A>> = vec![];

        edges.extend(self.iter_entering_edges().cloned());
        edges.extend(self.iter_internal_edges().cloned());
        edges.extend(rhs.iter_internal_edges().cloned());
        edges.extend(rhs.iter_leaving_edges().cloned());

        // connect lhs::leaving to rhs::entering
        edges.extend(self.iter_leaving_edges().flat_map(|leaving| {
            rhs.iter_entering_edges().map(|entering| {
                Edge {
                    from: leaving.from,
                    action: entering.action.clone() + leaving.action.clone(),
                    to: entering.to
                }
            })
        }));

        self.edges = edges;
    }

    pub fn merge(mut self, mut rhs: Self) -> Self {
        let merged = Self::default();
        rhs.offset(self.offset + self.count);

        merged
    }
}

/// Offset the fragment
impl std::ops::AddAssign<usize> for Graph {
    fn add_assign(&mut self, rhs: usize) {
        self.offset(rhs)
    }
}

/// Append two fragments
impl std::ops::Add<Graph> for Graph {
    type Output = Self;

    fn add(mut self, rhs: Graph) -> Self::Output {
        self.append(rhs);
        self
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Node {
    Start,
    Internal(usize),
    End
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
        match self {
            Node::Internal(value) => *value += rhs,
            _ => {}
        }
    }
}

#[derive(Clone)]
pub struct Edge<C, A> {
    from: Node,
    condition: C,
    action: Action<A>,
    to: Node,
}


impl<C,A> std::ops::Mul<Self> for Edge<C,A> 
where C: Set
{
    type Output = Vec<CrossEdge<C, A>>;

    fn mul(self, rhs: Self) -> Vec<CrossEdge<C, A>> 
    {
        let left = CrossEdge {
            from: CrossNode::Left(left.from),
            // E1 - E2
            condition: left.condition.clone().difference(right.condition.clone()),
            to: CrossNode::Left(left.to)
        };

        let right = CrossEdge {
            from: CrossNode::Right(right.from),
            // E2 - E1
            condition: right.condition.clone().difference(left.condition.clone()),
            to: CrossNode::Right(right.to)
        };

        let shared = CrossEdge {
            from: CrossNode::Shared(left.from, right.from),
            condition: right.condition.clone().intersect(left.condition.clone()),
            to: CrossNode::Shared(left.to, right.to)
        };
        
        [left, right, shared].into_iter().filter(|e| !e.is_empty()).collect()
    }
}

impl std::ops::AddAssign<usize> for Edge {
    fn add_assign(&mut self, rhs: usize) {
        self.to += rhs;
        self.from += rhs;
    }
}

#[derive(Default)]
pub struct Action<A>(Vec<A>);

//////////////////////////////
/// Cross graph operations ///
//////////////////////////////

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum CrossNode {
    Left(Node),
    Right(Node),
    Shared(Node, Node)
}

pub struct CrossEdge<C,A> 
where C: Set
{
    from: CrossNode,
    condition: C,
    action: Action<A>,
    to: CrossNode
}


impl<C,A> CrossEdge<C,A> 
where C: Set
{
    pub fn are_similar(&self, rhs: &Self) -> bool {
        self.from == rhs.from && self.to == rhs.to
    }

    pub fn is_empty(&self) -> bool {
        self.condition.is_empty()
    }

    /// Merge two edges with the same source and destination.
    /// 
    /// Actions are added
    /// Conditions are intersected
    pub fn merge(self: Self, rhs: Self) -> Self {
        if self.are {
            panic!("not mergeable");
        }

        Self {
            from: self.from,
            to: self.to,
            action: self.action + rhs.action,
            condition: self.condition.intersect(rhs.condition)
        }
    }
}