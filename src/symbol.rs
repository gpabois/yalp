use std::hash::Hash;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum SymbolKind {
    Normal,
    EOS,
    Start,
    Epsilon,
}

/// Defines a symbol
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Symbol<'s> {
    /// *Unique* identifier of the symbol
    pub id: &'s str,
    /// Set the symbol as terminal
    terminal: bool,
    kind: SymbolKind,
}

impl std::fmt::Display for Symbol<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl<'s> Symbol<'s> {
    /// Creates a new symbol
    pub fn new(id: &'s str, terminal: bool) -> Self {
        Self {
            id,
            terminal,
            kind: SymbolKind::Normal,
        }
    }

    #[inline(always)]
    pub fn is_terminal(&self) -> bool {
        self.terminal
    }

    #[inline(always)]
    pub fn is_eos(&self) -> bool {
        matches!(self.kind, SymbolKind::EOS)
    }

    #[inline(always)]
    pub fn is_start(&self) -> bool {
        matches!(self.kind, SymbolKind::Start)
    }

    #[inline(always)]
    pub fn is_epsilon(&self) -> bool {
        matches!(self.kind, SymbolKind::Epsilon)
    }

    /// Creates and end-of-stream token ($)
    pub fn eos() -> Self {
        Self {
            id: "<eos>",
            terminal: true,
            kind: SymbolKind::EOS,
        }
    }

    /// Creates a start symbol (S)
    pub fn start() -> Self {
        Self {
            id: "<start>",
            terminal: false,
            kind: SymbolKind::Start,
        }
    }

    /// Creates an epsilon symbol (ε)
    ///
    /// This is used for empty rule such as A -> ε ;
    pub fn epsilon() -> Self {
        Self {
            id: "<eps>",
            terminal: true,
            kind: SymbolKind::Epsilon,
        }
    }
}

impl<'s> Hash for Symbol<'s> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
