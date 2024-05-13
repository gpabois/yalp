use super::{RuleDef, Symbol, Rule};

#[derive(Debug, Clone)]
pub enum GrammarError<'s> {
    UnknownSymbol(&'s str),
    SymbolWithSameId(&'s str),
}

impl std::fmt::Display for GrammarError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GrammarError::UnknownSymbol(sym) => write!(f, "Unknown symbol: {}", sym),
            GrammarError::SymbolWithSameId(sym) => write!(f, "A symbol with the same identifier ({}) is already defined", sym),
        }
    }
}

pub type GrammarResult<'s, T> = Result<T, GrammarError<'s>>;

#[derive(Default)]
pub struct Grammar<'sid> {
    pub rules: Vec<RuleDef<'sid>>,
    pub symbols: Vec<Symbol<'sid>>
}

impl<'sid> Grammar<'sid> {
    /// Add a non-terminal symbol in the grammar.
    pub fn add_non_terminal_symbol(&mut self, id: &'sid str) -> GrammarResult<'sid, &mut Self> {
        if self.get_symbol(id).is_some() {
            Err(GrammarError::SymbolWithSameId(id))
        }
        else {
            self.symbols.push(Symbol::new(id, false));
            Ok(self)
        }
    }

    /// Add a terminal symbol in the grammar.
    pub fn add_terminal_symbol(&mut self, id: &'sid str) -> GrammarResult<'sid, &mut Self> {
        if self.get_symbol(id).is_some() {
            Err(GrammarError::SymbolWithSameId(id))
        }
        else {
            self.symbols.push(Symbol::new(id, true));
            Ok(self)
        }     
    }

    /// Get a symbol based on its id.
    pub fn get_symbol(&self, id: &str) -> Option<&Symbol<'sid>> {
        self.symbols.iter().find(|s| s.id == id)
    }

    /// Add a new rule
    pub fn add_rule<I>(&mut self, lhs: &'sid str, rhs: I) -> GrammarResult<'sid, &mut Self> where I: IntoIterator<Item=&'sid str> {
        let rule = RuleDef::new(
            self.rules.len(), 
            self.get_symbol(lhs)
                .and_then(|sym| Some(sym.id))
                .ok_or(GrammarError::UnknownSymbol(lhs))?,
            rhs.into_iter()
                .map(|id| self
                        .get_symbol(id)
                        .and_then(|sym| Some(sym.id))
                        .ok_or(GrammarError::UnknownSymbol(id))
                )
                .collect::<GrammarResult<'sid, Vec<_>>>()?
        );
        self.rules.push(rule);
        Ok(self)
    }

    #[inline(always)]
    fn borrow_rule(&self, def: &RuleDef<'sid>) -> Rule<'sid, '_> {
        Rule {
            id: def.id,
            lhs: self.get_symbol(def.lhs).unwrap(),
            rhs: def.rhs.clone().into_iter().map(|s| self.get_symbol(s)).collect::<Option<Vec<_>>>().unwrap()
        }
    }

    /// Iterate over all rules of the grammar
    pub fn iter_rules<'sym>(&'sym self) -> impl Iterator<Item=Rule<'sid, 'sym>> {
        self.rules.iter().map(|r| self.borrow_rule(r))
    }
}
