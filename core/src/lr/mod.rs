use std::marker::PhantomData;

use crate::{
    lexer::traits::Lexer,
    parser::traits::{Ast, Parser},
    ItemSetId,
};
use crate::{ErrorKind, YalpError, YalpResult};

mod action;
mod graph;
mod table;
mod transition;

use action::*;
use graph::*;
pub use table::*;
use transition::*;

pub use self::traits::LrTable;

pub type StateId = ItemSetId;

pub struct LrParser<'table, Ast, Table>
where
    Table: LrTable,
    Ast: crate::prelude::Ast,
{
    table: &'table Table,
    pht: PhantomData<Ast>,
}

impl<'table, Ast, Table> LrParser<'table, Ast, Table>
where
    Table: LrTable,
    Ast: crate::prelude::Ast,
{
    pub fn new(table: &'table Table) -> Self {
        Self {
            table,
            pht: PhantomData,
        }
    }
}

impl<'table, Table, Ast, Error> Parser<Error> for LrParser<'table, Ast, Table>
where
    Error: Clone,
    Ast: crate::prelude::Ast,
    Table: LrTable,
{
    type Ast = Ast;

    fn parse<L: Lexer<Error>>(&self, lexer: &mut L) -> YalpResult<Self::Ast, Error>
    where
        Self::Ast: From<L::Token>,
    {
        let mut states: Vec<StateId> = vec![0];
        let mut stack: Vec<Ast> = Vec::default();
        let mut cursor = lexer.next();

        loop {
            let mut state = states.last().copied().unwrap();

            let (symbol, tok) = match &cursor {
                None => (self.rules.eos(), None),
                Some(Ok(tok)) => (self.rules.try_get_symbol_by_id(tok.symbol_id())?, Some(tok)),
                Some(Err(err)) => return Err(err.clone()),
            };

            let action = self.table.action(state, &symbol).ok_or_else(|| {
                YalpError::new(
                    ErrorKind::unexpected_symbol(
                        symbol.id,
                        self.table.iter_terminals(state).map(|s| s.id.to_string()),
                    ),
                    None,
                )
            })?;

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
                                            &node.symbol_id().to_string(),
                                            vec![expected_symbol.id],
                                        ),
                                        None,
                                    ))
                                } else {
                                    Ok(())
                                }
                            })?;

                        states.truncate(states.len().saturating_sub(consume));
                        state = states.last().copied().unwrap();

                        let goto = self.table.goto(state, &rule.lhs).ok_or_else(|| {
                            YalpError::new(
                                ErrorKind::unexpected_symbol(
                                    &rule.lhs.id,
                                    self.table
                                        .iter_non_terminals(state)
                                        .map(|s| s.id.to_string()),
                                ),
                                None,
                            )
                        })?;

                        states.push(goto);

                        let reducer = self.reducers.get(*rule_id).unwrap();
                        reducer.reduce(rule, drained.into())
                    }?;

                    if ast.symbol_id() != rule.lhs.id {
                        return Err(YalpError::new(
                            ErrorKind::unexpected_symbol(ast.symbol_id(), vec![rule.lhs.id]),
                            None,
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
        fixtures::{FIXTURE_LR0_GRAMMAR, FIXTURE_LR1_GRAMMAR},
        lexer::fixtures::{lexer_fixture_lr0, lexer_fixture_lr1},
        NoCustomError,
    };

    use super::{LrParser, LrTable};

    #[test]
    pub fn test_lr0_grammar_table_building() {
        let table = LrTable::build::<0, _, NoCustomError>(&FIXTURE_LR0_GRAMMAR)
            .expect("cannot build table");
        println!("{}", table);
    }

    #[test]
    pub fn test_lr1_grammar_table_building() {
        let table = LrTable::build::<1, _, NoCustomError>(&FIXTURE_LR1_GRAMMAR)
            .expect("cannot build table");
        println!("{}", table);
    }

    #[test]
    pub fn test_lr0_parser() {
        let table = LrTable::build::<0, _, NoCustomError>(&FIXTURE_LR0_GRAMMAR)
            .expect("cannot build table");

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
        let table = LrTable::build::<1, _, NoCustomError>(&FIXTURE_LR1_GRAMMAR)
            .expect("cannot build table");

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
                AstNodeReducer,
            ],
        );

        let ast = parser.parse(&mut lexer).unwrap();
        println!("{:#?}", ast);
    }
}
