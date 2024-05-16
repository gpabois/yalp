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

#[derive(PartialEq)]
pub struct Grammar<'sid> {
    pub rules: Vec<RuleDef<'sid>>,
    pub symbols: Vec<Symbol<'sid>>,
}

impl Default for Grammar<'_> {
    fn default() -> Self {
        Self {
            rules: Default::default(),
            // $ is end of stream.
            symbols: vec![Symbol::root(), Symbol::eos()],
        }
    }
}

impl<'sid> Grammar<'sid> {
    pub fn eos(&self) -> &Symbol<'sid> {
        self.symbols.iter().find(|s| s.eos).unwrap()
    }
    /// Add a non-terminal symbol in the grammar.
    pub fn add_non_terminal_symbol(&mut self, id: &'sid str) -> GrammarResult<'sid, &mut Self> {
        if self.try_get_symbol(id).is_some() {
            Err(GrammarError::SymbolWithSameId(id))
        } else {
            self.symbols.push(Symbol::new(id, false));
            Ok(self)
        }
    }

    /// Add a terminal symbol in the grammar.
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
    /// Panics if not symbol match the ID.
    pub fn sym(&self, id: &str) -> &Symbol<'sid> {
        self.try_get_symbol(id).unwrap()
    }

    /// Add a new rule
    pub fn add_rule<I>(&mut self, lhs: &'sid str, rhs: I) -> GrammarResult<'sid, &mut Self>
    where
        I: IntoIterator<Item = &'sid str>,
    {
        let rule = RuleDef::new(
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

        self.rules.push(rule);

        Ok(self)
    }

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

    /// Iterate over all rules of the grammar
    pub fn iter_rules<'sym>(&'sym self) -> impl Iterator<Item = Rule<'sid, 'sym>> {
        self.rules.iter().map(|r| self.borrow_rule(r))
    }

    pub fn iter_terminal_symbols(&self) -> impl Iterator<Item = &Symbol<'sid>> {
        self.symbols.iter().filter(|sym| sym.terminal)
    }


    pub fn iter_non_terminal_symbols(&self) -> impl Iterator<Item = &Symbol<'sid>> {
        self.symbols.iter().filter(|sym| !sym.terminal)
    }
}
