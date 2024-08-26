pub enum Expr {
    Sequence(Sequence),
    Either(Either),
    Quantified(Quantified),
    Leaf(Leaf),
}

impl IntoGraph for Expr {
    fn into_graph(self) -> Graph {
        match self {
            Expr::Sequence(seq) => seq.into_graph(),
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

impl IntoGraph for Sequence {
    fn into_graph(self) -> GraphFragment {
        self.into_iter()
            .map(IntoGraphFragment::into_graph)
            .reduce(|a, b| a + b)
            .unwrap_or_default()
    }
}

/// A1 | A2 | ... | An
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

/// (A1)
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

/// A{n,m} or A+, or A?, or A*
pub struct Quantified(Box<Expr>, Quantifier);

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
