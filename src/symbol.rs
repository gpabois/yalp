use std::hash::Hash;

/// Defines a symbol
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Symbol<'s> {
    /// *Unique* identifier of the symbol
    pub id: &'s str,
    /// Set the symbol as terminal
    pub terminal: bool,
    pub eos: bool,
}

impl<'s> Symbol<'s> {
    pub fn new(id: &'s str, terminal: bool) -> Self {
        Self {
            id,
            terminal,
            eos: false,
        }
    }

    pub fn eos() -> Self {
        Self {
            id: "<eos>",
            terminal: true,
            eos: true,
        }
    }
}

impl<'s> Hash for Symbol<'s> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
