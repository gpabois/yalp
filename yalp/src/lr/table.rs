use prettytable::Table as PtTable;
use std::collections::HashMap;

use crate::traits::IntoRef;
use crate::traits::SymbolSliceable as _;
use crate::{grammar::traits::Grammar, ItemSetId, RuleSet, Symbol};

use super::{Action, Graph, LrParserError, LrResult, Transition};

#[derive(Debug, PartialEq, Eq)]
pub struct Row<'sid, 'sym> {
    actions: HashMap<&'sym Symbol<'sid>, Action>,
    goto: HashMap<&'sym Symbol<'sid>, ItemSetId>,
}

impl<'sid, 'sym> Row<'sid, 'sym> {
    pub fn new<A, G>(actions: A, goto: G) -> Self
    where
        A: IntoIterator<Item = (&'sym Symbol<'sid>, Action)>,
        G: IntoIterator<Item = (&'sym Symbol<'sid>, ItemSetId)>,
    {
        Self {
            actions: actions.into_iter().collect(),
            goto: goto.into_iter().collect(),
        }
    }

    pub fn action(&self, symbol: &'sym Symbol<'sid>) -> Option<&Action> {
        self.actions.get(symbol)
    }

    pub fn goto(&self, symbol: &'sym Symbol<'sid>) -> Option<ItemSetId> {
        self.goto.get(symbol).copied()
    }
}

impl<'sid, 'g> Row<'sid, 'g> {
    fn from_transition_lr1<const K: usize>(
        transition: Transition<'sid, 'g, '_, '_, K>,
        symbols: &'g [Symbol<'sid>],
    ) -> LrResult<'g, 'sid, Self> {
        let mut actions = HashMap::<&'g Symbol<'sid>, Action>::default();
        let mut goto = HashMap::<&'g Symbol<'sid>, ItemSetId>::default();

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
            if actions.contains_key(&sym) && matches!(actions[sym], Action::Reduce(_)) {
                return Err(LrParserError::ShiftReduceConflict {
                    state: transition.from.id,
                    symbol: sym,
                    conflict: [action, actions[sym]],
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
        transition: Transition<'sid, 'g, '_, '_, K>,
        symbols: &'g [Symbol<'sid>],
    ) -> LrResult<'g, 'sid, Self> {
        let mut actions = HashMap::<&'g Symbol<'sid>, Action>::default();
        let mut goto = HashMap::<&'g Symbol<'sid>, ItemSetId>::default();

        for (sym, action) in transition
            .edges
            .iter()
            .filter(|(sym, _)| sym.is_terminal())
            .filter(|(sym, _)| !sym.is_eos())
            .filter(|(sym, _)| !sym.is_epsilon())
            .map(|(sym, set)| (*sym, Action::Shift(set.id)))
        {
            // Shift/reduce conflict
            if actions.contains_key(&sym) && matches!(actions[sym], Action::Reduce(_)) {
                return Err(LrParserError::ShiftReduceConflict {
                    state: transition.from.id,
                    symbol: sym,
                    conflict: [action, actions[sym]],
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
        transition: Transition<'sid, 'g, '_, '_, K>,
        symbols: &'g [Symbol<'sid>],
    ) -> LrResult<'g, 'sid, Self> {
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
    rows: Vec<Row<'sym, 'sid>>,
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
                            .get(sym)
                            .map(ToString::to_string)
                            .unwrap_or_default()
                    }))
                    .chain(self.symbols.iter_non_terminals().map(|sym| {
                        row.goto
                            .get(sym)
                            .map(ToString::to_string)
                            .unwrap_or_default()
                    }))
                    .collect(),
            );
        }

        write!(f, "{}", table)
    }
}

impl<'sid, 'sym> LrTable<'sid, 'sym>
where
    'sid: 'sym,
{
    fn iter(&self) -> impl Iterator<Item = &Row<'sym, 'sid>> {
        self.rows.iter()
    }

    pub fn get(&self, state_id: usize) -> Option<&Row<'sid, 'sym>> {
        self.rows.get(state_id)
    }

    fn from_graph<const K: usize>(
        graph: &Graph<'sid, 'sym, '_, K>,
        symbols: &'sym [Symbol<'sid>],
    ) -> LrResult<'sym, 'sid, Self> {
        Ok(Self {
            symbols,
            rows: graph
                .iter_transitions()
                .map(|t| Row::from_transition(t, symbols))
                .collect::<LrResult<'sym, 'sid, Vec<_>>>()?,
        })
    }

    /// Build a LR Table parser from a grammar.
    pub fn build<const K: usize, G>(grammar: &'sym G) -> LrResult<'sym, 'sid, Self>
    where
        G: Grammar<'sid, 'sym>,
        &'sym G: IntoRef<'sym, [Symbol<'sid>]>,
    {
        let rules = RuleSet::new(grammar);

        let mut graph = Graph::<K>::new(&rules);
        graph.build()?;

        LrTable::from_graph(&graph, grammar.as_symbol_slice())
    }
}
