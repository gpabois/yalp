use std::collections::HashMap;

use crate::{Grammar, RuleSet, Symbol};

use self::{graph::Graph, table::Transition};

mod graph;
mod item;
mod table;

pub enum Action {
    Shift(usize),
    Reduce(usize),
    Accept,
}

pub struct Row<'sym, 'sid> {
    actions: HashMap<&'sym Symbol<'sid>, Action>,
    goto: HashMap<&'sym Symbol<'sid>, usize>,
}

impl<'sym, 'sid> Row<'sym, 'sid> {
    pub fn from_transition(
        transition: Transition<'sid, 'sym, '_, '_>,
        grammar: &'sym Grammar<'sid>,
    ) -> Self {
        if transition.from.has_item_reaching_eos() {
            Row {
                actions: [(grammar.eos(), Action::Accept)].into_iter().collect(),
                goto: HashMap::default(),
            }
        } else if transition.from.has_terminating_item() {
            let rule_id = transition.from.get_terminating_rule();

            Row {
                actions: grammar
                    .iter_terminal_symbols()
                    .map(|sym| (sym, Action::Reduce(rule_id)))
                    .collect(),
                goto: HashMap::default(),
            }
        } else {
            Row {
                actions: transition
                    .edges
                    .iter()
                    .filter(|(sym, _)| sym.terminal)
                    .map(|(sym, set)| (*sym, Action::Shift(set.id)))
                    .collect(),
                goto: transition
                    .edges
                    .iter()
                    .filter(|(sym, _)| !sym.terminal)
                    .map(|(sym, set)| (*sym, set.id))
                    .collect(),
            }
        }
    }
}

pub struct Table<'sym, 'sid>(Vec<Row<'sym, 'sid>>);

impl<'sym, 'sid> Table<'sym, 'sid> {
    fn from_graph(graph: &Graph<'sid, 'sym, '_>, grammar: &'sym Grammar<'sid>) -> Self {
        Self(
            graph
                .iter_transitions()
                .map(|t| Row::from_transition(t, grammar))
                .collect(),
        )
    }

    /// Build a LR Table parser from a grammar.
    pub fn build(grammar: &'sym Grammar<'sid>) -> Self {
        let rules = RuleSet::new(grammar);

        let mut graph = Graph::new(&rules);
        graph.build();

        Table::from_graph(&graph, grammar)
    }
}
