use super::atomic::Atomic;

pub trait IntoGraphFragment {
    fn into_graph_fragment(self) -> GraphFragment;
}

#[derive(Clone)]
pub struct Leaf {
    kind: Atomic,
}

impl Leaf {
    pub fn intersect(&self, rhs: &Self) -> Self {
        Self {
            kind: self.kind.intersect(&rhs.kind),
        }
    }
}

impl IntoGraphFragment for Leaf {
    fn into_graph_fragment(self) -> GraphFragment {
        let mut fragment = GraphFragment::default();
        let node = fragment.add_node();
        fragment.entering(node, self);
        fragment
    }
}

pub enum Expr {
    Sequence(Sequence),
    Either(Either),
    Quantified(Quantified),
    Leaf(Leaf),
}

impl IntoGraphFragment for Expr {
    fn into_graph_fragment(self) -> GraphFragment {
        match self {
            Expr::Sequence(seq) => seq.into_graph_fragment(),
            Expr::Either(_) => todo!(),
            Expr::Quantified(_) => todo!(),
            Expr::Leaf(_) => todo!(),
        }
    }
}

/// A1..An
pub struct Sequence(Vec<Expr>);

impl IntoIterator for Sequence {
    type Item = Expr;
    type IntoIter = <Vec<Expr> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl IntoGraphFragment for Sequence {
    fn into_graph_fragment(self) -> GraphFragment {
        self.into_iter()
            .map(IntoGraphFragment::into_graph_fragment)
            .reduce(|a, b| a + b)
            .unwrap_or_default()
    }
}

pub struct Either(Vec<Expr>);

impl IntoIterator for Either {
    type Item = Expr;
    type IntoIter = <Vec<Expr> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl IntoGraphFragment for Either {
    fn into_graph_fragment(self) -> GraphFragment {
        self.into_iter()
            .map(IntoGraphFragment::into_graph_fragment)
            .reduce(GraphFragment::merge)
            .unwrap_or_default()
    }
}

pub struct Group(Box<Expr>, Option<String>);

pub enum Quantifier {
    /// *, or {0,}
    Wild,
    /// ?, or {0,1}
    Optional,
    /// {n,}
    RangeFrom(usize),
    /// {,m}
    RangeTo(usize),
    /// {n,m}
    Range(usize, usize),
}
pub struct Quantified(Box<Expr>, Quantifier);

pub struct EnteringEdge {
    leaf: Leaf,
    to: usize,
}

#[derive(Clone)]
pub struct RightArrowSet(Vec<RightArrow>);

impl RightArrowSet {
    pub fn zip(left: Self, right: Self) -> impl Iterator<Item = (RightArrow, RightArrow)> {
        left.0.into_iter().flat_map(move |left| {
            right
                .clone()
                .0
                .into_iter()
                .map(move |right| (left.clone(), right.clone()))
        })
    }
    pub fn merge(left: Self, right: Self) -> MergedRightEdgeSet {
        Self::zip(left, right).map(|(left, right)| {
            let left_leaf = left.leaf.difference(&right.leaf);
            let right_leaf = right.leaf.difference(&left.leaf);
            let common_leaf = left.leaf.intersect(&right.leaf);
        });
    }
}

pub struct MergedRightEdgeSet {
    pub left: RightArrowSet,
    pub right: RightArrowSet,
    pub common: Vec<MergedRightArrow>,
}

pub struct MergedRightArrow {
    pub leaf: Leaf,
    // (left, right)
    pub merge: [usize; 2],
}

#[derive(Clone)]
pub struct RightArrow {
    leaf: Leaf,
    to: usize,
}

impl From<EnteringEdge> for RightArrow {
    fn from(value: EnteringEdge) -> Self {
        Self {
            leaf: value.leaf,
            to: value.to,
        }
    }
}

impl EnteringEdge {
    pub fn intersect(&self, rhs: &Self) -> Leaf {
        self.leaf.intersect(&rhs.leaf)
    }
}

impl std::ops::AddAssign<usize> for EnteringEdge {
    fn add_assign(&mut self, rhs: usize) {
        self.to += rhs
    }
}

pub struct LeavingEdge {
    from: usize,
}

impl LeavingEdge {
    pub fn connect(&self, rhs: &EnteringEdge) -> Edge {
        Edge {
            from: self.from,
            leaf: rhs.leaf.clone(),
            to: rhs.to,
        }
    }
}

impl std::ops::AddAssign<usize> for LeavingEdge {
    fn add_assign(&mut self, rhs: usize) {
        self.from += rhs
    }
}

pub struct Edge {
    from: usize,
    leaf: Leaf,
    to: usize,
}

impl std::ops::AddAssign<usize> for Edge {
    fn add_assign(&mut self, rhs: usize) {
        self.to += rhs;
        self.from += rhs;
    }
}

#[derive(Default)]
pub struct GraphFragment {
    entering: Vec<EnteringEdge>,
    leaving: Vec<LeavingEdge>,
    edges: Vec<Edge>,

    offset: usize,
    count: usize,
}

impl GraphFragment {
    pub fn add_node(&mut self) -> usize {
        let node = self.count;
        self.count += 1;
        node
    }

    pub fn connect(&mut self, from: usize, leaf: Leaf, to: usize) {
        self.edges.push(Edge { from, to, leaf })
    }

    pub fn entering(&mut self, to: usize, leaf: Leaf) {
        self.entering.push(EnteringEdge { leaf, to })
    }

    pub fn leaving(&mut self, from: usize) {
        self.leaving.push(LeavingEdge { from })
    }

    pub fn offset(&mut self, n: usize) {
        self.offset += n;
        self.entering.iter_mut().for_each(|edge| *edge += n);
        self.edges.iter_mut().for_each(|edge| *edge += n);
        self.leaving.iter_mut().for_each(|edge| *edge += n);
    }

    pub fn append(&mut self, mut rhs: Self) {
        rhs += self.offset + self.count;

        self.count += rhs.count;
        self.edges.extend(rhs.edges);
        // connect lhs::leaving to rhs::entering
        self.edges.extend(self.leaving.iter().flat_map(|leaving| {
            rhs.entering
                .iter()
                .map(|entering| leaving.connect(entering))
        }));
        // leaving of the summed fragments is the rhs's leaving.
        self.leaving = rhs.leaving;
    }

    pub fn merge(mut self, mut rhs: Self) -> Self {
        let merged = Self::default();
        rhs.offset(self.offset + self.count);

        merged
    }
}

/// Offset the fragment
impl std::ops::AddAssign<usize> for GraphFragment {
    fn add_assign(&mut self, rhs: usize) {
        self.offset(rhs)
    }
}

/// Append two fragments
impl std::ops::Add<GraphFragment> for GraphFragment {
    type Output = Self;

    fn add(mut self, rhs: GraphFragment) -> Self::Output {
        self.append(rhs);
        self
    }
}
