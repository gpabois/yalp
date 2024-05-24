use std::hash::Hash;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum SymbolKind {
    Terminal,
    NonTerminal,
    EOS,
    Start,
    Epsilon,
}

/// The identifier of a symbol.
pub struct SymbolId(str);

/// Defines a symbol
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Symbol<'s> {
    /// *Unique* identifier of the symbol
    pub id: &'s str,
    kind: SymbolKind,
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

    pub trait IntoRef<'a, T: ?Sized> {
        fn into_ref(self) -> &'a T;
    }

    impl<'a, T: ?Sized> IntoRef<'a, T> for &'a T {
        fn into_ref(self) -> &'a T {
            self
        }
    }

    /// A trait to implement common methods for object holding symbols.
    pub trait SymbolSliceable<'sid, 'sym, 'a>
    where
        'sid: 'sym,
        Self: 'a,
        &'a Self: IntoRef<'sym, [Symbol<'sid>]>,
    {
        fn sym(&'a self, id: &str) -> &'sym Symbol<'sid> {
            self.into_ref()
                .iter()
                .find(|sym| sym.id == id)
                .expect(&format!("the grammar does not include symbol {}", id))
        }

        fn eos(&'a self) -> &'sym Symbol<'sid> {
            self.into_ref()
                .iter()
                .find(|sym| Symbol::is_eos(sym))
                .expect("the grammar does not include <eos> terminal.")
        }

        fn start(&'a self) -> &'sym Symbol<'sid> {
            self.into_ref()
                .iter()
                .find(|sym| Symbol::is_start(sym))
                .expect("the grammar does not include <start> symbol.")
        }

        fn epsilon(&'a self) -> &'sym Symbol<'sid> {
            self.into_ref()
                .iter()
                .find(|sym| Symbol::is_epsilon(sym))
                .expect("the grammar does not include <eps> terminal.")
        }

        fn get_symbol_by_id(&'a self, id: &str) -> Option<&'sym Symbol<'sid>> {
            self.into_ref().iter().find(|sym| sym.id == id)
        }

        fn iter_terminals(&'a self) -> impl Iterator<Item = &'sym Symbol<'sid>> {
            self.into_ref().iter().filter(|sym| sym.is_terminal())
        }

        fn iter_non_terminals(&'a self) -> impl Iterator<Item = &'sym Symbol<'sid>> {
            self.into_ref().iter().filter(|sym| !sym.is_terminal())
        }

        fn as_symbol_slice(&'a self) -> &'sym [Symbol<'sid>] {
            self.into_ref()
        }
    }

    impl<'sid, 'sym, 'a, T> SymbolSliceable<'sid, 'sym, 'a> for T
    where
        T: 'a + ?Sized,
        &'a T: IntoRef<'sym, [Symbol<'sid>]>,
        'sid: 'sym,
    {
    }
}
