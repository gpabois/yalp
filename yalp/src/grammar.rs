use crate::traits::IntoRef;

use super::{RuleDef, Symbol};

#[derive(Debug, Clone)]
pub enum GrammarError<'s> {
    UnknownSymbol(&'s str),
    SymbolWithSameId(&'s str),
}

impl std::fmt::Display for GrammarError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GrammarError::UnknownSymbol(sym) => write!(f, "Unknown symbol: {}", sym),
            GrammarError::SymbolWithSameId(sym) => write!(
                f,
                "A symbol with the same identifier ({}) is already defined",
                sym
            ),
        }
    }
}

pub type GrammarResult<'s, T> = Result<T, GrammarError<'s>>;

pub mod traits {

    use crate::{
        traits::{IntoRef, SymbolSliceable},
        Rule, RuleDef, Symbol,
    };

    pub trait Grammar<'sid, 'g>
    where
        'sid: 'g,
        Self: 'g + AsRef<[RuleDef<'sid>]> + SymbolSliceable<'sid, 'g, 'g>,
        &'g Self: IntoRef<'g, [Symbol<'sid>]>,
    {
        const NB_RULES: usize;
        const NB_SYMBOLS: usize;

        fn iter_rules(&'g self) -> impl Iterator<Item = Rule<'sid, 'g>> {
            self.as_ref()
                .iter()
                .enumerate()
                .map(move |(rule_id, rule_def)| Rule {
                    id: rule_id,
                    lhs: self.sym(rule_def.lhs),
                    rhs: rule_def.rhs.iter().map(|id| self.sym(*id)).collect(),
                })
        }
    }
}

#[derive(Debug, PartialEq)]
/// A grammar
///
/// A grammar requires a rule to produce the START symbol, which must have EOS as its end.
///
/// # Example
///
/// For the following grammar :
///
/// ```grammar
/// 1. <start> := E <eos>
/// 2. E := E * B
/// 3. E := E + B
/// 4. E := B
/// 5. B := 0
/// 6. B := 1
/// ```
///
/// ```
/// use crate::grammar;
///
/// let grammar = grammar! {
///     symbols: [
///         term!(*),
///         term!(+),
///         term!(0),
///         term!(1),
///         nterm(E),
///         nterm(B)
///     ],
///     rules: [
///         rule!(START => "E" EOS),
///         rule!("E" => "E" "*" "B"),
///         rule!("E" => "E" "+" "B"),
///         rule!("E" => "B"),
///         rule!("B" => "0"),
///         rule!("B" => "1")
///     ]
/// };
///
/// ```
pub struct Grammar<'sid, const NB_SYMBOLS: usize, const NB_RULES: usize> {
    rules: [RuleDef<'sid>; NB_RULES],
    symbols: [Symbol<'sid>; NB_SYMBOLS],
}

impl<'sid, const NB_SYMBOLS: usize, const NB_RULES: usize> Grammar<'sid, NB_SYMBOLS, NB_RULES> {
    pub const fn new(
        symbols: [Symbol<'sid>; NB_SYMBOLS],
        rules: [RuleDef<'sid>; NB_RULES],
    ) -> Self {
        Self { rules, symbols }
    }
}

impl<'sid, 'g, const NB_SYMBOLS: usize, const NB_RULES: usize> IntoRef<'g, [Symbol<'sid>]>
    for &'g Grammar<'sid, NB_SYMBOLS, NB_RULES>
where
    'sid: 'g,
{
    fn into_ref(self) -> &'g [Symbol<'sid>] {
        &self.symbols
    }
}

impl<'sid, const NB_SYMBOLS: usize, const NB_RULES: usize> AsRef<[RuleDef<'sid>]>
    for Grammar<'sid, NB_SYMBOLS, NB_RULES>
{
    fn as_ref(&self) -> &[RuleDef<'sid>] {
        &self.rules
    }
}

impl<'sid, 'g, const NB_SYMBOLS: usize, const NB_RULES: usize> traits::Grammar<'sid, 'g>
    for Grammar<'sid, NB_SYMBOLS, NB_RULES>
where
    'sid: 'g,
{
    const NB_RULES: usize = NB_RULES;
    const NB_SYMBOLS: usize = NB_SYMBOLS;
}
