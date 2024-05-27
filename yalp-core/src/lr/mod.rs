use std::marker::PhantomData;

use crate::grammar::traits::Grammar;
use crate::rule::traits::RuleReducer;
use crate::token::traits::Token;
use crate::traits::SymbolSlice as _;
use crate::{
    lexer::traits::Lexer,
    parser::{traits::Ast, traits::Parser},
    ItemSetId, RuleSet,
};
use crate::{YalpError, ErrorKind, YalpResult};

mod action;
pub mod codegen;
mod graph;
mod table;
mod transition;

pub use action::*;
use graph::*;
pub use table::*;
use transition::*;

pub struct LrParser<'sid, 'sym, 'table, 'reducers, Node, Table, Reducer, Error>
where
    Node: Ast,
    Table: self::traits::LrTable,
    Reducer: RuleReducer<'sid, Error, Ast = Node>
{
    rules: RuleSet<'sid, 'sym>,
    table: &'table Table,
    reducers: &'reducers [Reducer],
    _phantom: PhantomData<(Node, Error)>,
}

impl<'sid, 'g, 'table, 'reducers, Node, Table, Reducer, Error>
    LrParser<'sid, 'g, 'table, 'reducers, Node, Table, Reducer, Error>
where
    Node: Ast,
    Table: self::traits::LrTable,
    Reducer: RuleReducer<'sid, Error, Ast = Node>
{
    pub fn new<G>(grammar: &'g G, table: &'table Table, reducers: &'reducers [Reducer]) -> Self
    where
        G: Grammar<'sid>,
    {
        if reducers.len() != grammar.iter_rules().count() {
            panic!(
                "{}",
                &format!(
                    "the number of reducers must match the number of grammar rules {}.",
                    &grammar.iter_rules().count().to_string()
                )
            )
        }

        Self {
            rules: RuleSet::new(grammar),
            table,
            reducers,
            _phantom: PhantomData,
        }
    }
}

impl<'sid, 'sym, 'table, 'reducers, Node, Table, Reducer, Error> Parser<Error>
    for LrParser<'sid, 'sym, 'table, 'reducers, Node, Table, Reducer, Error>
where
    Error: Clone,
    Node: Ast,
    Table: self::traits::LrTable,
    Reducer: RuleReducer<'sid, Error, Ast = Node>
{
    type Ast = Node;

    fn parse<L: Lexer<Error>>(&self, lexer: &mut L) -> YalpResult<Self::Ast, Error>
    where
        Self::Ast: From<L::Token>,
    {
        let mut states: Vec<ItemSetId> = vec![0];
        let mut stack: Vec<Node> = Vec::default();

        let mut cursor = lexer.next();

        loop {
            let mut state = states.last().copied().unwrap();

            let (symbol, tok) = match &cursor {
                None => (self.rules.eos(), None),
                Some(Ok(tok)) => (
                    self.rules
                        .try_get_symbol_by_id(tok.symbol_id())?,
                    Some(tok),
                ),
                Some(Err(err)) => return Err(err.clone()),
            };

            let action = self
                .table
                .action(state, &symbol)
                .ok_or_else(|| YalpError::new(ErrorKind::unexpected_symbol(
                    symbol.id,
                    self.table.iter_terminals(state).map(|s| s.id.to_string())
                ), None))?;
    
            println!("#{} {} :: {}", state, symbol, action);
            match action {
                // Push the new terminal on top of the stack
                // Shift to tne given state.
                Action::Shift(next_state_id) => {
                    if !symbol.is_eos() {
                        stack.push(tok.cloned().unwrap().into());
                        cursor = lexer.next();
                    }
                    states.push(*next_state_id);
                }

                // Reduce by the given rule
                // Consume LHS's length number of symbols
                Action::Reduce(rule_id) => {
                    let rule = self.rules.borrow_rule(*rule_id);
                    let consume = rule.rhs.len();

                    let ast = {
                        let drained = stack.drain(stack.len().saturating_sub(consume)..);
                        drained
                            .as_slice()
                            .iter()
                            .zip(rule.rhs.iter())
                            .try_for_each(|(node, expected_symbol)| {
                                if node.symbol_id() != expected_symbol.id {
                                    Err(YalpError::new(
                                        ErrorKind::unexpected_symbol(
                                            &node.symbol_id().to_string(), vec![expected_symbol.id]), 
                                        None
                                    ))
                                } else {
                                    Ok(())
                                }
                            })?;

                        states.truncate(states.len().saturating_sub(consume));
                        state = states.last().copied().unwrap();

                        let goto = self
                            .table
                            .goto(state, &rule.lhs)
                            .ok_or_else(|| YalpError::new(
                                ErrorKind::unexpected_symbol(
                                    &rule.lhs.id, 
                                    self.table.iter_non_terminals(state).map(|s| s.id.to_string())
                                ), 
                                None
                            ))?;
                            
                        states.push(goto);

                        let reducer = self.reducers.get(*rule_id).unwrap();
                        reducer.reduce(rule, drained.into())
                    }?;

                    if ast.symbol_id() != rule.lhs.id {
                        return Err(YalpError::new(
                            ErrorKind::unexpected_symbol(
                                ast.symbol_id() ,
                                vec![rule.lhs.id]), 
                            None
                        ));
                    }

                    stack.push(ast);
                }
                Action::Accept => {
                    return Ok(stack.pop().unwrap());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::AstNodeReducer, fixtures::{FIXTURE_LR0_GRAMMAR, FIXTURE_LR1_GRAMMAR}, lexer::fixtures::{lexer_fixture_lr0, lexer_fixture_lr1}, traits::Parser as _, NoCustomError
    };

    use super::{LrParser, LrTable};

    #[test]
    pub fn test_lr0_grammar_table_building() {
        let table = LrTable::build::<0, _, NoCustomError>(&FIXTURE_LR0_GRAMMAR).expect("cannot build table");
        println!("{}", table);
    }

    #[test]
    pub fn test_lr1_grammar_table_building() {
        let table = LrTable::build::<1, _, NoCustomError>(&FIXTURE_LR1_GRAMMAR).expect("cannot build table");
        println!("{}", table);
    }

    #[test]
    pub fn test_lr0_parser() {
        let table = LrTable::build::<0, _, NoCustomError>(&FIXTURE_LR0_GRAMMAR).expect("cannot build table");

        let mut lexer = lexer_fixture_lr0("1 + 1 * 0 * 1 * 1".chars());

        let parser = LrParser::new(
            &FIXTURE_LR0_GRAMMAR,
            &table,
            &[
                AstNodeReducer, 
                AstNodeReducer, 
                AstNodeReducer, 
                AstNodeReducer, 
                AstNodeReducer, 
                AstNodeReducer,
            ],
        );

        let ast = parser.parse(&mut lexer).unwrap();
        println!("{:#?}", ast);
    }

    #[test]
    pub fn test_lr1_parser() {
        let table = LrTable::build::<1, _, NoCustomError>(&FIXTURE_LR1_GRAMMAR).expect("cannot build table");

        let mut lexer = lexer_fixture_lr1("n + n".chars());
        let parser = LrParser::new(
            &FIXTURE_LR1_GRAMMAR,
            &table,
            &[
                AstNodeReducer,
                AstNodeReducer,
                AstNodeReducer,
                AstNodeReducer,
                AstNodeReducer,
                AstNodeReducer
            ],
        );

        let ast = parser.parse(&mut lexer).unwrap();
        println!("{:#?}", ast);
    }
}
