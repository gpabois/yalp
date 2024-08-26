use pb_bnf::symbol::Symbol;
use prettytable::Table as PtTable;
use std::{collections::HashMap, u16};

use crate::{
    syntax::{PrepSyntax, SymbolSet, Syntax},
    ErrorKind, ItemSetId, YalpError, YalpResult,
};

use super::{Action, Graph, Transition};

pub mod traits {
    use crate::lr::Action;

    pub trait LrTable {
        fn action<'table>(&'table self, state: usize, symbol: &str) -> Option<&'table Action>;
        fn goto(&self, state: usize, symbol: &str) -> Option<usize>;
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Row {
    actions: HashMap<SymbolId, Action>,
    goto: HashMap<SymbolId, ItemSetId>,
}

impl Row {
    pub fn new<A, G>(actions: A, goto: G) -> Self
    where
        A: IntoIterator<Item = (SymbolId, Action)>,
        G: IntoIterator<Item = (SymbolId, ItemSetId)>,
    {
        Self {
            actions: actions.into_iter().collect(),
            goto: goto.into_iter().collect(),
        }
    }

    pub fn action<'a, 'b>(&'a self, symbol_id: &SymbolId) -> Option<&'a Action>
    where
        'b: 'a,
    {
        self.actions.get(symbol_id)
    }

    pub fn goto(&self, symbol_id: &SymbolId) -> Option<ItemSetId> {
        self.goto.get(symbol_id).copied()
    }
}

impl Row {
    fn from_transition_lr1<'syntax, const K: usize, Error>(
        transition: Transition<'syntax, '_, '_, K>,
        symbols: &SymbolSet<'syntax>,
        map: &SymbolMap,
    ) -> YalpResult<Self, Error> {
        let mut actions = HashMap::<Symbol<'syntax>, Action>::default();
        let mut goto = HashMap::<Symbol<'syntax>, ItemSetId>::default();

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
                return Err(YalpError::new(
                    ErrorKind::ShiftReduceConflict {
                        state: transition.from.id,
                        symbol: sym.to_owned(),
                        conflict: [action, actions[&sym]],
                    },
                    None,
                ));
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

    fn from_transition_lr0<'syntax, const K: usize, Error>(
        transition: Transition<'syntax, '_, '_, K>,
        symbols: &SymbolSet<'syntax>,
        map: &SymbolMap,
    ) -> YalpResult<Self, Error> {
        let mut actions = HashMap::<Symbol<'syntax>, Action>::default();
        let mut goto = HashMap::<Symbol<'syntax>, ItemSetId>::default();

        for (sym, action) in transition
            .edges
            .iter()
            .filter(|(sym, _)| sym.is_terminal())
            .filter(|(sym, _)| !sym.is_eos())
            .map(|(sym, set)| (*sym, Action::Shift(set.id)))
        {
            // Shift/reduce conflict
            if actions.contains_key(&sym) && matches!(actions[&sym], Action::Reduce(_)) {
                return Err(YalpError::new(
                    ErrorKind::ShiftReduceConflict {
                        state: transition.from.id,
                        symbol: sym.to_owned(),
                        conflict: [action, actions[&sym]],
                    },
                    None,
                ));
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
    pub fn from_transition<'syntax, const K: usize, Error>(
        transition: Transition<'syntax, '_, '_, K>,
        symbols: &SymbolSet<'syntax>,
        map: &SymbolMap,
    ) -> YalpResult<Self, Error> {
        if K == 0 {
            Self::from_transition_lr0(transition, symbols, map)
        } else if K == 1 {
            Self::from_transition_lr1(transition, symbols, map)
        } else {
            Err(YalpError::new(ErrorKind::UnsupportedAlgorithm, None))
        }
    }
}

struct SymbolMap {
    terminals: Vec<String>,
    non_terminals: Vec<String>,
}

type SymbolId = u16;

impl From<SymbolSet<'_>> for SymbolMap {
    fn from(value: SymbolSet<'_>) -> Self {
        Self::new(
            value.terminals.iter().map(|sym| sym.deref()),
            value.non_terminals.iter().map(|sym| sym.deref()),
        )
    }
}

impl SymbolMap {
    pub fn new(
        terminals: impl Iterator<Item = String>,
        non_terminals: impl Iterator<Item = String>,
    ) -> Self {
        Self {
            terminals: terminals.collect(),
            non_terminals: non_terminals.collect(),
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.terminals
            .iter()
            .chain(self.non_terminals.iter())
            .map(|sym| sym.as_str())
    }

    pub fn get_internal_id(&self, symbol_id: &str) -> Option<SymbolId> {
        self.iter()
            .enumerate()
            .find(|(_, sym)| sym == symbol_id)
            .map(|(iid, _)| iid)
    }
}

#[derive(PartialEq)]
pub struct LrTable {
    /// An internal symbol mapping
    symbols: SymbolMap,
    /// The table rows
    rows: Vec<Row>,
}

impl std::fmt::Debug for LrTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        <Self as std::fmt::Display>::fmt(self, f)
    }
}

impl<'syntax> std::fmt::Display for LrTable {
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

impl traits::LrTable for LrTable {
    fn action<'table>(&'table self, state: usize, symbol: &str) -> Option<&'table Action> {
        self.rows.get(state).and_then(|row| row.action(symbol))
    }

    fn goto(&self, state: usize, symbol: &str) -> Option<usize> {
        self.rows.get(state).and_then(|row| row.goto(symbol))
    }
}

impl LrTable {
    fn iter(&self) -> impl Iterator<Item = &Row> {
        self.rows.iter()
    }

    fn from_graph<'syntax, 'gen, const K: usize, Error>(
        graph: &Graph<'syntax, 'gen, K>,
        syntax: &'gen PrepSyntax<'syntax>,
    ) -> YalpResult<Self, Error> {
        let symbols = Symbol::from(syntax.symbols);

        Ok(Self {
            symbols,
            rows: graph
                .iter_transitions()
                .map(|t| Row::from_transition(t, &syntax.symbols, &symbols))
                .collect::<YalpResult<Vec<_>, Error>>()?,
        })
    }

    /// Build a LR Table parser from a grammar.
    pub fn build<const K: usize, G, Error>(syntax: &Syntax) -> YalpResult<Self, Error> {
        let rules = PrepSyntax::from(syntax);

        let mut graph = Graph::<K>::new(&rules);
        graph.build()?;

        LrTable::from_graph(&graph, &rules)
    }
}
