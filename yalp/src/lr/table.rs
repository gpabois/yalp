use prettytable::Table as PtTable;
use std::collections::HashMap;

use crate::{grammar::traits::Grammar, traits::SymbolSlice as _, ItemSetId, RuleSet, Symbol};

use super::{Action, Graph, LrParserError, LrResult, Transition};

pub mod traits {
    use crate::{lr::Action, Symbol};

    pub trait LrTable {
        fn action<'a, 'b>(&'a self, state: usize, symbol: &Symbol<'b>) -> Option<&'a Action>
        where
            'b: 'a;
        fn goto(&self, state: usize, symbol: &Symbol<'_>) -> Option<usize>;

        /// The number of rows in the table.
        fn len(&self) -> usize;
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Row<'sid> {
    actions: HashMap<Symbol<'sid>, Action>,
    goto: HashMap<Symbol<'sid>, ItemSetId>,
}

impl<'sid> Row<'sid> {
    pub fn new<A, G>(actions: A, goto: G) -> Self
    where
        A: IntoIterator<Item = (Symbol<'sid>, Action)>,
        G: IntoIterator<Item = (Symbol<'sid>, ItemSetId)>,
    {
        Self {
            actions: actions.into_iter().collect(),
            goto: goto.into_iter().collect(),
        }
    }

    pub fn action<'a, 'b>(&'a self, symbol: &Symbol<'b>) -> Option<&'a Action>
    where
        'b: 'a,
    {
        self.actions.get(symbol)
    }

    pub fn goto(&self, symbol: &Symbol<'sid>) -> Option<ItemSetId> {
        self.goto.get(symbol).copied()
    }
}

impl<'sid> Row<'sid> {
    fn from_transition_lr1<const K: usize>(
        transition: Transition<'sid, '_, '_, K>,
        symbols: &[Symbol<'sid>],
    ) -> LrResult<Self> {
        let mut actions = HashMap::<Symbol<'sid>, Action>::default();
        let mut goto = HashMap::<Symbol<'sid>, ItemSetId>::default();

        if transition.from.has_item_reaching_eos() {
            actions.insert(symbols.eos(), Action::Accept);
        }

        for (sym, action) in transition
            .edges
            .iter()
            .filter(|(sym, _)| sym.is_terminal())
            .map(|(sym, set)| (*sym, Action::Shift(set.id)))
        {
            // Shift/reduce conflict
            if actions.contains_key(&sym) && matches!(actions[&sym], Action::Reduce(_)) {
                return Err(LrParserError::ShiftReduceConflict {
                    state: transition.from.id,
                    symbol: sym.to_owned(),
                    conflict: [action, actions[&sym]],
                });
            }

            actions.insert(sym, action);
        }

        goto.extend(
            transition
                .edges
                .iter()
                .filter(|(sym, _)| !sym.is_terminal())
                .map(|(sym, set)| (*sym, set.id)),
        );

        actions.extend(
            transition
                .from
                .iter_exhausted_items()
                .map(|item| (item.lookaheads[0], Action::Reduce(item.rule.id))),
        );

        Ok(Self::new(actions, goto))
    }

    fn from_transition_lr0<const K: usize>(
        transition: Transition<'sid, '_, '_, K>,
        symbols: &[Symbol<'sid>],
    ) -> LrResult<Self> {
        let mut actions = HashMap::<Symbol<'sid>, Action>::default();
        let mut goto = HashMap::<Symbol<'sid>, ItemSetId>::default();

        for (sym, action) in transition
            .edges
            .iter()
            .filter(|(sym, _)| sym.is_terminal())
            .filter(|(sym, _)| !sym.is_eos())
            .filter(|(sym, _)| !sym.is_epsilon())
            .map(|(sym, set)| (*sym, Action::Shift(set.id)))
        {
            // Shift/reduce conflict
            if actions.contains_key(&sym) && matches!(actions[&sym], Action::Reduce(_)) {
                return Err(LrParserError::ShiftReduceConflict {
                    state: transition.from.id,
                    symbol: sym.to_owned(),
                    conflict: [action, actions[&sym]],
                });
            }

            actions.insert(sym, action);
        }

        goto.extend(
            transition
                .edges
                .iter()
                .filter(|(sym, _)| !sym.is_terminal())
                .map(|(sym, set)| (*sym, set.id)),
        );

        if transition.from.has_item_reaching_eos() {
            actions.insert(symbols.eos(), Action::Accept);
        }

        if transition.from.has_exhausted_items() {
            let rule_id = transition.from.get_exhausted_rule();
            actions.extend(
                symbols
                    .iter_terminals()
                    .map(|sym| (sym, Action::Reduce(rule_id))),
            );
        }

        Ok(Self::new(actions, goto))
    }
    pub fn from_transition<const K: usize>(
        transition: Transition<'sid, '_, '_, K>,
        symbols: &[Symbol<'sid>],
    ) -> LrResult<Self> {
        if K == 0 {
            Self::from_transition_lr0(transition, symbols)
        } else if K == 1 {
            Self::from_transition_lr1(transition, symbols)
        } else {
            Err(LrParserError::UnsupportedLrRank)
        }
    }
}

#[derive(PartialEq)]
pub struct LrTable<'sid, 'sym> {
    symbols: &'sym [Symbol<'sid>],
    rows: Vec<Row<'sid>>,
}

impl std::fmt::Debug for LrTable<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        <Self as std::fmt::Display>::fmt(self, f)
    }
}

impl<'sym, 'sid> std::fmt::Display for LrTable<'sym, 'sid> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = PtTable::new();

        table.add_row(
            ["#"]
                .into_iter()
                .chain(
                    self.symbols
                        .iter_terminals()
                        .chain(self.symbols.iter_non_terminals())
                        .map(|sym| sym.id),
                )
                .collect(),
        );

        for (id, row) in self.iter().enumerate() {
            table.add_row(
                [id.to_string()]
                    .into_iter()
                    .chain(self.symbols.iter_terminals().map(|sym| {
                        row.actions
                            .get(&sym)
                            .map(ToString::to_string)
                            .unwrap_or_default()
                    }))
                    .chain(self.symbols.iter_non_terminals().map(|sym| {
                        row.goto
                            .get(&sym)
                            .map(ToString::to_string)
                            .unwrap_or_default()
                    }))
                    .collect(),
            );
        }

        write!(f, "{}", table)
    }
}

impl traits::LrTable for LrTable<'_, '_> {
    fn action<'a, 'b>(&'a self, state: usize, symbol: &Symbol<'b>) -> Option<&'a Action>
    where
        'b: 'a,
    {
        self.rows.get(state).and_then(|row| row.action(symbol))
    }

    fn goto(&self, state: usize, symbol: &Symbol<'_>) -> Option<usize> {
        self.rows.get(state).and_then(|row| row.goto(symbol))
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

impl<'sid, 'sym> LrTable<'sid, 'sym>
where
    'sid: 'sym,
{
    fn iter(&self) -> impl Iterator<Item = &Row<'sid>> {
        self.rows.iter()
    }

    pub fn get(&self, state_id: usize) -> Option<&Row<'sid>> {
        self.rows.get(state_id)
    }

    fn from_graph<const K: usize>(
        graph: &Graph<'sid, 'sym, '_, K>,
        symbols: &'sym [Symbol<'sid>],
    ) -> LrResult<Self> {
        Ok(Self {
            symbols,
            rows: graph
                .iter_transitions()
                .map(|t| Row::from_transition(t, symbols))
                .collect::<LrResult<Vec<_>>>()?,
        })
    }

    /// Build a LR Table parser from a grammar.
    pub fn build<const K: usize, G>(grammar: &'sym G) -> LrResult<Self>
    where
        G: Grammar<'sid>,
    {
        let rules = RuleSet::new(grammar);

        let mut graph = Graph::<K>::new(&rules);
        graph.build()?;

        LrTable::from_graph(&graph, grammar.as_symbol_slice())
    }
}

/// Module to generate static tables.
pub mod codegen {
    use crate::{lr::Action, Symbol};

    pub struct LrTableRow<const NB_TERMS: usize, const NB_NTERMS: usize> {
        actions: [(&'static str, Option<Action>); NB_TERMS],
        goto: [(&'static str, Option<usize>); NB_NTERMS],
    }

    impl<const NB_TERMS: usize, const NB_NTERMS: usize> LrTableRow<NB_TERMS, NB_NTERMS> {
        pub fn action<'a, 'b>(&'a self, symbol: &Symbol<'b>) -> Option<&'a Action> {
            self.actions
                .iter()
                .find(|(id, _)| symbol.id == *id)
                .and_then(|(_, act)| act.as_ref())
        }

        pub fn goto(&self, symbol: &Symbol<'_>) -> Option<usize> {
            self.goto
                .iter()
                .find(|(id, _)| symbol.id == *id)
                .and_then(|(_, goto)| *goto)
        }
    }

    pub struct LrTable<const NB_STATES: usize, const NB_TERMS: usize, const NB_NTERMS: usize> {
        rows: [LrTableRow<NB_TERMS, NB_NTERMS>; NB_STATES],
    }
}
