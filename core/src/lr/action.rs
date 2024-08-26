use crate::syntax::RuleId;

use super::StateId;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Action {
    Shift(StateId),
    Reduce(RuleId),
    Accept,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Shift(to) => write!(f, "s{}", to),
            Action::Reduce(to) => write!(f, "r{}", to),
            Action::Accept => write!(f, "acc"),
        }
    }
}
