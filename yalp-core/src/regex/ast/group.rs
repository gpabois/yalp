use crate::{charset::CharSet, dfa::IntoGraph};

use super::{Action, Expr};


/// A capturing group : example: (A1)
pub struct Group {
    expr: Box<Expr>, 
    id: Option<String>
}

impl Group {
    /// New anonymous capturing group
    pub fn new_anonymous(expr: Expr) -> Self {
        Self {
            expr: Box::new(expr),
            id: None
        }
    }

    pub fn new<S: ToString>(id: S, expr: Expr) -> Self {
        Self {
            expr: Box::new(expr),
            id: Some(id.to_string())
        }
    }
}

impl IntoGraph<CharSet, Action> for Group {
    fn into_graph(self) -> crate::dfa::Graph<CharSet, Action> {
        let mut sub = self.expr.into_graph();
        sub
            .iter_mut_entering_edges()
            .for_each(|edge| edge.actions.push(Action::PushGroup { id: self.id.clone() }));

        sub
            .iter_mut_leaving_edges()
            .for_each(|edge| edge.actions.push(Action::PopGroup));

        sub
    }
}