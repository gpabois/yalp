/// Defines a symbol
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Symbol<'s> {
    /// *Unique* identifier of the symbol
    pub id: &'s str,
    /// Set the symbol as terminal
    pub terminal: bool
}

impl<'s> Symbol<'s> {
    pub fn new(id: &'s str, terminal: bool) -> Self {
        Self {id, terminal}
    }
}
