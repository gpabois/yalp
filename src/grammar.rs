use super::{Rule, RuleDef, Symbol};

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

#[derive(Debug, PartialEq)]
/// A grammar
/// 
/// A grammar requires a rule to produce the <start> token, which must have <eos> token
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
/// use crate::Gammar;
/// 
/// let mut grammar = Grammar::default();
/// 
/// grammar
///     .add_terminal_symbol("*")?
///     .add_terminal_symbol("+")?
///     .add_terminal_symbol("0")?
///     .add_terminal_symbol("1")?
///     .add_non_terminal_symbol("E")?
///     .add_non_terminal_symbol("B")?;
/// ```
pub struct Grammar<'sid> {
    rules: Vec<RuleDef<'sid>>,
    symbols: Vec<Symbol<'sid>>,
}

impl Default for Grammar<'_> {
    fn default() -> Self {
        Self {
            rules: Default::default(),
            symbols: vec![Symbol::start(), Symbol::eos(), Symbol::epsilon()],
        }
    }
}

impl<'sid> Grammar<'sid> {
    /// Returns the end-of-stream symbol (<eos>) of the grammar.
    pub fn eos(&self) -> &Symbol<'sid> {
        self.symbols.iter().find(|s| s.is_eos()).unwrap()
    }

    /// Returns the start symbol (<start>) of the grammar.
    pub fn start(&self) -> &Symbol<'sid> {
        self.symbols.iter().find(|s| s.is_start()).unwrap()
    }

    pub fn epsilon(&self) -> &Symbol<'sid> {
        self.symbols.iter().find(|s| s.is_epsilon()).unwrap()
    }

    /// Add a non-terminal symbol in the grammar.
    /// 
    /// Returns an error if a symbol with the same id already exists.
    pub fn add_non_terminal_symbol(&mut self, id: &'sid str) -> GrammarResult<'sid, &mut Self> {
        if self.try_get_symbol(id).is_some() {
            Err(GrammarError::SymbolWithSameId(id))
        } else {
            self.symbols.push(Symbol::new(id, false));
            Ok(self)
        }
    }

    /// Add a terminal symbol in the grammar.
    /// 
    /// Returns an error if a symbol with the same id already exists.
    pub fn add_terminal_symbol(&mut self, id: &'sid str) -> GrammarResult<'sid, &mut Self> {
        if self.try_get_symbol(id).is_some() {
            Err(GrammarError::SymbolWithSameId(id))
        } else {
            self.symbols.push(Symbol::new(id, true));
            Ok(self)
        }
    }

    /// Get a symbol based on its id.
    pub fn try_get_symbol(&self, id: &str) -> Option<&Symbol<'sid>> {
        self.symbols.iter().find(|s| s.id == id)
    }

    /// Returns the symbol behind the ID
    /// 
    /// # Panics
    /// Panics if no symbol match the ID.
    pub fn sym(&self, id: &str) -> &Symbol<'sid> {
        self.try_get_symbol(id).unwrap()
    }

    /// Add a new rule
    /// 
    /// Returns an error if a symbol defined in the rule does not exist within the grammar.
    pub fn add_rule<I>(&mut self, lhs: &'sid str, rhs: I) -> GrammarResult<'sid, &mut Self>
    where
        I: IntoIterator<Item = &'sid str>,
    {
        let mut rule = RuleDef::new(
            self.rules.len(),
            self.try_get_symbol(lhs)
                .map(|sym| sym.id.as_ref())
                .ok_or(GrammarError::UnknownSymbol(lhs))?,
            rhs.into_iter()
                .map(|id| {
                    self.try_get_symbol(id)
                        .map(|sym| sym.id)
                        .ok_or(GrammarError::UnknownSymbol(id))
                })
                .collect::<GrammarResult<'sid, Vec<_>>>()?,
        );

        if rule.rhs.is_empty() {
            rule.rhs.push(self.epsilon().id)
        }

        self.rules.push(rule);

        Ok(self)
    }


    /// Iterate over all rules of the grammar
    pub fn iter_rules<'sym>(&'sym self) -> impl Iterator<Item = Rule<'sid, 'sym>> {
        self.rules.iter().map(|r| self.borrow_rule(r))
    }

    pub fn iter_terminal_symbols(&self) -> impl Iterator<Item = &Symbol<'sid>> {
        self.symbols.iter().filter(|sym| sym.is_terminal())
    }

    pub fn iter_non_terminal_symbols(&self) -> impl Iterator<Item = &Symbol<'sid>> {
        self.symbols.iter().filter(|sym| !sym.is_terminal())
    }
}

impl<'sid> Grammar<'sid> {
    #[inline(always)]
    fn borrow_rule(&self, def: &RuleDef<'sid>) -> Rule<'sid, '_> {
        Rule {
            id: def.id,
            lhs: self.try_get_symbol(def.lhs).unwrap(),
            rhs: def
                .rhs
                .clone()
                .into_iter()
                .map(|s| self.try_get_symbol(s))
                .collect::<Option<Vec<_>>>()
                .unwrap(),
        }
    }
}