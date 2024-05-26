use std::hash::Hash;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum SymbolKind {
    Terminal,
    NonTerminal,
    EOS,
    Start,
    Epsilon,
}

#[derive(Debug, Clone)]
pub struct OwnedSymbol {
    pub id: String,
    kind: SymbolKind,
}

impl OwnedSymbol {
    pub fn borrow(&self) -> Symbol<'_> {
        Symbol {
            id: &self.id,
            kind: self.kind,
        }
    }
}

impl std::fmt::Display for OwnedSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// Defines a symbol
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Symbol<'s> {
    /// *Unique* identifier of the symbol
    pub id: &'s str,
    kind: SymbolKind,
}

impl<'s> Symbol<'s> {
    pub fn to_owned(&self) -> OwnedSymbol {
        OwnedSymbol {
            id: self.id.to_owned(),
            kind: self.kind.to_owned(),
        }
    }
}

pub const START: &str = "<start>";
pub const EOS: &str = "<eos>";

impl std::fmt::Display for Symbol<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl<'s> Symbol<'s> {
    /// Creates a new symbol
    pub const fn new(id: &'s str, terminal: bool) -> Self {
        Self {
            id,
            kind: if terminal {
                SymbolKind::Terminal
            } else {
                SymbolKind::NonTerminal
            },
        }
    }

    pub const fn term(id: &'s str) -> Self {
        Self::new(id, true)
    }

    pub const fn nterm(id: &'s str) -> Self {
        Self::new(id, false)
    }

    #[inline(always)]
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.kind,
            SymbolKind::EOS | SymbolKind::Epsilon | SymbolKind::Terminal
        )
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

    /// Creates and end-of-stream token ($, or <eos>)
    pub const fn eos() -> Self {
        Self {
            id: "<eos>",
            kind: SymbolKind::EOS,
        }
    }

    /// Creates a start symbol (S)
    pub const fn start() -> Self {
        Self {
            id: "<start>",
            kind: SymbolKind::Start,
        }
    }

    /// Creates an epsilon symbol (ε)
    ///
    /// This is used for empty rule such as A -> ε ;
    pub const fn epsilon() -> Self {
        Self {
            id: "<eps>",
            kind: SymbolKind::Epsilon,
        }
    }
}

impl<'s> Hash for Symbol<'s> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

pub mod traits {
    use crate::Symbol;

    /// A trait to implement common methods for object holding symbols.
    pub trait SymbolSlice<'sid>
    where
        Self: AsRef<[Symbol<'sid>]>,
    {
        fn sym(&self, id: &str) -> Symbol<'sid> {
            self.as_ref()
                .iter()
                .find(|sym| sym.id == id)
                .copied()
                .unwrap_or_else(|| panic!("the grammar does not include symbol {}", id))
        }

        fn eos(&self) -> Symbol<'sid> {
            self.as_ref()
                .iter()
                .find(|sym| Symbol::is_eos(sym))
                .copied()
                .expect("the grammar does not include <eos> terminal.")
        }

        fn start(&self) -> Symbol<'sid> {
            self.as_ref()
                .iter()
                .find(|sym| Symbol::is_start(sym))
                .copied()
                .expect("the grammar does not include <start> symbol.")
        }

        fn epsilon(&self) -> Symbol<'sid> {
            self.as_ref()
                .iter()
                .find(|sym| Symbol::is_epsilon(sym))
                .copied()
                .expect("the grammar does not include <eps> terminal.")
        }

        fn get_symbol_by_id(&self, id: &str) -> Option<Symbol<'sid>> {
            self.as_ref().iter().find(|sym| sym.id == id).copied()
        }

        fn iter_terminals<'a>(&'a self) -> impl Iterator<Item = Symbol<'sid>> + 'a
        where
            'sid: 'a,
        {
            self.as_ref()
                .iter()
                .filter(|sym| sym.is_terminal())
                .copied()
        }

        fn iter_non_terminals<'a>(&'a self) -> impl Iterator<Item = Symbol<'sid>> + 'a
        where
            'sid: 'a,
        {
            self.as_ref()
                .iter()
                .filter(|sym| !sym.is_terminal())
                .copied()
        }

        fn as_symbol_slice(&self) -> &[Symbol<'sid>] {
            self.as_ref()
        }
    }

    impl<'sid, T> SymbolSlice<'sid> for T where T: AsRef<[Symbol<'sid>]> {}
}
