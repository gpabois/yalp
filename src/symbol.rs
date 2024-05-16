use std::hash::Hash;

/// Defines a symbol
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Symbol<'s> {
    /// *Unique* identifier of the symbol
    pub id: &'s str,
    /// Set the symbol as terminal
    pub terminal: bool,
    pub eos: bool,
    pub root: bool
}

impl std::fmt::Display for Symbol<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl<'s> Symbol<'s> {
    pub fn new(id: &'s str, terminal: bool) -> Self {
        Self {
            id: id.into(),
            terminal,
            eos: false,
            root: false
        }
    }

    pub fn is_terminal(&self) -> bool {
        self.terminal
    }

    pub fn eos() -> Self {
        Self {
            id: "<eos>".into(),
            terminal: true,
            eos: true,
            root: false
        }
    }

    pub fn root() -> Self {
        Self {
            id: "<root>".into(),
            terminal: false,
            eos: false,
            root: true
        }  
    }
}

impl<'s> Hash for Symbol<'s> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
