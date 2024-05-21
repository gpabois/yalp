use prettytable::Table as PtTable;
use std::collections::HashMap;

use crate::{Grammar, ItemSetId, RuleSet, Symbol};

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
}

impl<'sym, 'sid> Row<'sym, 'sid> {
    fn from_transition_lr1<const K: usize>(
        transition: Transition<'sid, 'sym, '_, '_, K>,
        grammar: &'sym Grammar<'sid>,
    ) -> LrResult<'sym, 'sid, Self> {
        let mut actions = HashMap::<&'sym Symbol<'sid>, Action>::default();
        let mut goto = HashMap::<&'sym Symbol<'sid>, ItemSetId>::default();

        if transition.from.has_item_reaching_eos() {
            actions.insert(grammar.eos(), Action::Accept);
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
        transition: Transition<'sid, 'sym, '_, '_, K>,
        grammar: &'sym Grammar<'sid>,
    ) -> LrResult<'sym, 'sid, Self> {
        let mut actions = HashMap::<&'sym Symbol<'sid>, Action>::default();
        let mut goto = HashMap::<&'sym Symbol<'sid>, ItemSetId>::default();

        for (sym, action) in transition
            .edges
            .iter()
            .filter(|(sym, _)| sym.is_terminal())
            .filter(|(sym, _)| !sym.is_eos())
            .filter(|(sym, _)| !sym.is_epsilon())
            .inspect(|(sym, set)| {
                println!("{} - {} -> {}", transition.from.id, sym, set.id);
            })
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
            actions.insert(grammar.eos(), Action::Accept);
        }

        if transition.from.has_exhausted_items() {
            println!("{}", transition.from);
            let rule_id = transition.from.get_exhausted_rule();
            actions.extend(
                grammar
                    .iter_terminal_symbols()
                    .map(|sym| (sym, Action::Reduce(rule_id))),
            );
        }

        Ok(Self::new(actions, goto))
    }
    pub fn from_transition<const K: usize>(
        transition: Transition<'sid, 'sym, '_, '_, K>,
        grammar: &'sym Grammar<'sid>,
    ) -> LrResult<'sym, 'sid, Self> {
        if K == 0 {
            Self::from_transition_lr0(transition, grammar)
        } else if K == 1 {
            Self::from_transition_lr1(transition, grammar)
        } else {
            Err(LrParserError::UnsupportedLrRank)
        }
    }
}

#[derive(PartialEq)]
pub struct Table<'sym, 'sid> {
    grammar: &'sym Grammar<'sid>,
    rows: Vec<Row<'sym, 'sid>>,
}

impl std::fmt::Debug for Table<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n")?;
        <Self as std::fmt::Display>::fmt(&self, f)
    }
}

impl<'sym, 'sid> std::fmt::Display for Table<'sym, 'sid> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = PtTable::new();

        table.add_row(
            ["#"]
                .into_iter()
                .chain(
                    self.grammar
                        .iter_terminal_symbols()
                        .chain(self.grammar.iter_non_terminal_symbols())
                        .map(|sym| sym.id.as_ref()),
                )
                .collect(),
        );

        for (id, row) in self.iter().enumerate() {
            table.add_row(
                [id.to_string()]
                    .into_iter()
                    .chain(self.grammar.iter_terminal_symbols().map(|sym| {
                        row.actions
                            .get(sym)
                            .map(ToString::to_string)
                            .unwrap_or_default()
                    }))
                    .chain(self.grammar.iter_non_terminal_symbols().map(|sym| {
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

impl<'sym, 'sid> Table<'sym, 'sid> {
    pub fn new<I>(grammar: &'sym Grammar<'sid>, rows: I) -> Self
    where
        I: IntoIterator<Item = Row<'sym, 'sid>>,
    {
        Self {
            grammar,
            rows: rows.into_iter().collect(),
        }
    }

    fn iter(&self) -> impl Iterator<Item = &Row<'sym, 'sid>> {
        self.rows.iter()
    }

    fn from_graph<const K: usize>(
        graph: &Graph<'sid, 'sym, '_, K>,
        grammar: &'sym Grammar<'sid>,
    ) -> LrResult<'sym, 'sid, Self> {
        Ok(Self {
            grammar,
            rows: graph
                .iter_transitions()
                .map(|t| Row::from_transition(t, grammar))
                .collect::<LrResult<'sym, 'sid, Vec<_>>>()?,
        })
    }

    /// Build a LR Table parser from a grammar.
    pub fn build<const K: usize>(grammar: &'sym Grammar<'sid>) -> LrResult<'sym, 'sid, Self> {
        let rules = RuleSet::new(grammar);

        let mut graph = Graph::<K>::new(&rules);
        graph.build()?;

        Table::from_graph(&graph, grammar)
    }
}
