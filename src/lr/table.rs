use std::collections::HashMap;
use prettytable::Table as PtTable;

use super::{graph::Graph, item::ItemSet, ItemSetId, LrParserError, LrResult};
use crate::{array::Array, Grammar, Lookahead, RuleId, RuleSet, Symbol};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Action {
    Shift(ItemSetId),
    Reduce(RuleId),
    Accept,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Shift(to) => write!(f, "s{}", to),
            Action::Reduce(to) => write!(f, "r{}", to),
            Action::Accept => write!(f, "acc"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Row<'sid, 'sym, const K: usize> {
    actions: HashMap<Lookahead<'sid, 'sym, K>, Action>,
    goto: HashMap<&'sym Symbol<'sid>, ItemSetId>,
}

impl<'sid, 'sym, const K: usize> Row<'sid, 'sym, K> {
    pub fn new<A,G>(actions: A, goto: G) -> Self 
        where 
            A: IntoIterator<Item=(Lookahead<'sid, 'sym, K>, Action)>,
            G: IntoIterator<Item=(&'sym Symbol<'sid>, ItemSetId)>,
    {
        Self {
            actions: actions.into_iter().collect(),
            goto: goto.into_iter().collect()
        }
    }
}

impl<'sym, 'sid, const K: usize> Row<'sym, 'sid, K> 
{
    pub fn from_transition<const k: usize>(
        transition: Transition<'sid, 'sym, '_, '_, k>,
        grammar: &'sym Grammar<'sid>,
    ) -> LrResult<'sym, 'sid, Self> {
        let mut actions = HashMap::<&'sym Symbol<'sid>, Action>::default();
        let mut goto = HashMap::<&'sym Symbol<'sid>, ItemSetId>::default();

        if transition.from.has_item_reaching_eos() {
            actions.insert(grammar.eos(), Action::Accept);
        } else if transition.from.has_terminating_item() {
            let rule_id = transition.from.get_terminating_rule();
            actions.extend(grammar
                .iter_terminal_symbols()
                .map(|sym| (sym, Action::Reduce(rule_id)))
            );
        } else {
            for (sym, action) in transition
                .edges
                .iter()
                .filter(|(sym, _)| sym.terminal)
                .map(|(sym, set)| (*sym, Action::Shift(set.id))) {
                
                // Shift/reduce conflict
                if actions.contains_key(&sym) && matches!(actions[sym], Action::Reduce(_)) {
                    return Err(LrParserError::ShiftReduceConflict {
                        state: transition.from.id,
                        symbol: sym,
                        conflict: [
                            action,
                            actions[sym]
                        ],
                    })
                }

                actions.insert(sym, action);
            }

            goto.extend( transition
                .edges
                .iter()
                .filter(|(sym, _)| !sym.terminal)
                .map(|(sym, set)| (*sym, set.id))
            );
        }

        Ok(Self::new(actions, goto))
    }
}

#[derive(PartialEq)]
pub struct Table<'sym, 'sid, const K: usize>{
    grammar: &'sym Grammar<'sid>,
    rows: Vec<Row<'sym, 'sid, K>>
}

impl<const K: usize> std::fmt::Debug for Table<'_, '_, K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n")?;
        <Self as std::fmt::Display>::fmt(&self, f)
    }
}

impl<'sym, 'sid, const K: usize> std::fmt::Display for Table<'sym, 'sid, K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = PtTable::new();

        table.add_row(
            ["#"]
            .into_iter()
            .chain(            
                self
                .grammar
                .iter_terminal_symbols()
                .chain(self.grammar.iter_non_terminal_symbols())
                .map(|sym| sym.id.as_ref())
            )
            .collect()
        );

        for (id, row) in self.iter().enumerate() {
            table.add_row(
                [id.to_string()].into_iter()
                .chain(
                    self
                    .grammar
                    .iter_terminal_symbols()
                    .map(|sym| row.actions.get(sym).map(ToString::to_string).unwrap_or_default())
                )
                .chain(self.grammar.iter_non_terminal_symbols().map(|sym| row.goto.get(sym).map(ToString::to_string).unwrap_or_default()))
                .collect()
            );
        }

        write!(f, "{}", table)
    }
}

impl<'sym, 'sid, const K: usize> Table<'sym, 'sid, K> {
    pub fn new<I>(grammar: &'sym Grammar<'sid>, rows: I) -> Self where I: IntoIterator<Item=Row<'sym, 'sid, K>>{
        Self{
            grammar,
            rows: rows.into_iter().collect()
        }
    }

    fn iter(&self) -> impl Iterator<Item=&Row<'sym, 'sid, K>> {
        self.rows.iter()
    }

    fn from_graph(graph: &Graph<'sid, 'sym, '_, K>, grammar: &'sym Grammar<'sid>) -> LrResult<'sym, 'sid, Self> {
        Ok(Self{
            grammar,
            rows: graph
                .iter_transitions()
                .map(|t| Row::from_transition(t, grammar))
                .collect::<LrResult<'sym, 'sid, Vec<_>>>()?,
        })
    }

    /// Build a LR Table parser from a grammar.
    pub fn build(grammar: &'sym Grammar<'sid>) -> LrResult<'sym, 'sid, Self> {
        let rules = RuleSet::new(grammar);

        let mut graph = Graph::<K>::new(&rules);
        graph.build()?;

        Table::from_graph(&graph, grammar)
    }
}

pub struct Transition<'sid, 'sym, 'rule, 'set, const K: usize> {
    pub from: &'set ItemSet<'sid, 'sym, 'rule, K>,
    pub edges: Vec<(Lookahead<'sid, 'sym, K>, &'set ItemSet<'sid, 'sym, 'rule, K>)>,
}

impl<'sid, 'sym, 'rule, 'set, const K: usize> Transition<'sid, 'sym, 'rule, 'set, K> {
    pub fn new<I>(from: &'set ItemSet<'sid, 'sym, 'rule, K>, edges: I) -> Self
    where
        I: Iterator<Item = (Lookahead<'sid, 'sym, K>, &'set ItemSet<'sid, 'sym, 'rule, K>)>,
    {
        Self {
            from,
            edges: edges.collect(),
        }
    }
}

