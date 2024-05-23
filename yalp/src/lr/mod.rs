use std::fmt::Debug;

use crate::grammar::traits::Grammar;
use crate::sym::traits::SymbolSliceable as _;
use crate::token::traits::Token;
use crate::traits::IntoRef;
use crate::{
    lexer::{traits::Lexer, LexerError},
    parser::{traits::Ast, Parser},
    ItemSetId, RuleId, RuleReducer, RuleSet, Symbol,
};

mod action;
mod graph;
mod table;
mod transition;

pub use action::*;
use graph::*;
pub use table::*;
use transition::*;

#[derive(Debug)]
pub enum LrParserError<'sid, 'sym> {
    MissingRule(RuleId),
    MissingAction(ItemSetId, &'sym Symbol<'sid>),
    MissingGoto(ItemSetId, &'sym Symbol<'sid>),
    MissingState(ItemSetId),
    LexerError(LexerError),
    UnknownSymbol(String),
    UnexpectedSymbol {
        expected: &'sid str,
        got: String,
    },
    UnsupportedLrRank,
    ShiftReduceConflict {
        state: ItemSetId,
        symbol: &'sym Symbol<'sid>,
        conflict: [Action; 2],
    },
}

impl<'sid, 'sym> From<LexerError> for LrParserError<'sid, 'sym> {
    fn from(value: LexerError) -> Self {
        Self::LexerError(value)
    }
}

impl std::fmt::Display for LrParserError<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LrParserError::MissingRule(id) => write!(f, "missing rule #{}", id),
            LrParserError::MissingState(id) => write!(f, "missing state #{}", id),
            LrParserError::ShiftReduceConflict {
                state,
                symbol,
                conflict,
            } => write!(
                f,
                "shift/reduce conflict for symbol {}, (state: #{}) [{:?}]",
                symbol.id, state, conflict
            ),
            LrParserError::UnsupportedLrRank => write!(f, "cannot build LR table for K > 1."),
            LrParserError::LexerError(error) => write!(f, "{}", error),
            LrParserError::MissingAction(state_id, symbol) => write!(
                f,
                "missing action for terminal {} (state #{})",
                symbol, state_id
            ),
            LrParserError::MissingGoto(state_id, symbol) => write!(
                f,
                "missing goto for non-terminal {} (state #{})",
                symbol, state_id
            ),
            LrParserError::UnknownSymbol(symbol_id) => write!(f, "unknown symbol {}", symbol_id),
            LrParserError::UnexpectedSymbol { expected, got } => {
                write!(f, "unexpected symbol {}, expecting {}", got, expected)
            }
        }
    }
}

pub type LrResult<'sid, 'sym, T> = Result<T, LrParserError<'sid, 'sym>>;

pub struct LrParser<'sid, 'sym, 'table, 'reducers, Node>
where
    Node: Ast,
{
    rules: RuleSet<'sid, 'sym>,
    table: &'table Table<'sid, 'sym>,
    reducers: &'reducers [RuleReducer<'sid, Node>],
}

impl<'sid, 'g, 'table, 'reducers, Node> LrParser<'sid, 'g, 'table, 'reducers, Node>
where
    Node: Ast,
{
    pub fn new<G>(
        grammar: &'g G,
        table: &'table Table<'sid, 'g>,
        reducers: &'reducers [RuleReducer<'sid, Node>],
    ) -> Self
    where
        G: Grammar<'sid, 'g>,
        &'g G: IntoRef<'g, [Symbol<'sid>]>,
    {
        if reducers.len() != grammar.iter_rules().count() {
            panic!("the number of reducers must match the number of grammar rules.")
        }

        Self {
            rules: RuleSet::new(grammar),
            table,
            reducers,
        }
    }
}

impl<'sid, 'sym, 'table, 'reducers, Node> Parser for LrParser<'sid, 'sym, 'table, 'reducers, Node>
where
    Node: Ast,
{
    type Ast = Node;
    type Error = LrParserError<'sid, 'sym>;

    fn parse<L: Lexer>(&self, lexer: &mut L) -> Result<Self::Ast, Self::Error>
    where
        Self::Ast: From<L::Token>,
    {
        let mut state: ItemSetId = 0;
        let mut stack: Vec<Node> = Vec::default();

        while let Some(token_result) = lexer.next() {
            let token = token_result?;

            let symbol = self
                .rules
                .get_symbol_by_id(token.symbol_id())
                .ok_or_else(|| LrParserError::UnknownSymbol(token.symbol_id().to_string()))?;

            let row = self
                .table
                .get(state)
                .ok_or(LrParserError::MissingState(state))?;

            let action = row
                .action(symbol)
                .ok_or(LrParserError::MissingAction(state, symbol))?;

            match action {
                // Push the new terminal on top of the stack
                // Shift to tne given state.
                Action::Shift(next_state_id) => {
                    stack.push(token.into());
                    state = *next_state_id;
                }
                // Reduce by the given rule
                // Consume LHS's length number of symbols
                Action::Reduce(rule_id) => {
                    let rule = self.rules.get(*rule_id);
                    let consume = rule.rhs.len();

                    let ast = {
                        let drained = stack.drain(stack.len() - consume..);
                        drained
                            .as_slice()
                            .iter()
                            .zip(rule.rhs.iter())
                            .try_for_each(|(node, expected_symbol)| {
                                if node.symbol_id() != expected_symbol.id {
                                    Err(LrParserError::UnexpectedSymbol {
                                        expected: expected_symbol.id,
                                        got: node.symbol_id().to_string(),
                                    })
                                } else {
                                    Ok(())
                                }
                            })?;

                        let reducer = self.reducers.get(*rule_id).unwrap();
                        reducer(rule, drained)
                    };

                    if ast.symbol_id() != rule.lhs.id {
                        return Err(LrParserError::UnexpectedSymbol {
                            expected: rule.lhs.id,
                            got: ast.symbol_id().to_string(),
                        });
                    }

                    stack.push(ast);

                    state = row
                        .goto(rule.lhs)
                        .ok_or(LrParserError::MissingGoto(state, rule.lhs))?;
                }
                Action::Accept => {
                    return Ok(stack.pop().unwrap());
                }
            }
        }

        Err(LexerError::unexpected_end_of_stream(lexer.current_location()).into())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::AstNode,
        fixtures::{FIXTURE_LR0_GRAMMAR, FIXTURE_LR1_GRAMMAR},
        lexer::fixtures::lexer_fixture_lr0,
        parser::Parser as _,
    };

    use super::{LrParser, Table};

    #[test]
    pub fn test_lr0_grammar_table_building() {
        let table = Table::build::<0, _>(&FIXTURE_LR0_GRAMMAR).expect("cannot build table");
        println!("{}", table);
    }

    #[test]
    pub fn test_lr1_grammar_table_building() {
        let table = Table::build::<1, _>(&FIXTURE_LR1_GRAMMAR).expect("cannot build table");
        println!("{}", table);
    }

    #[test]
    pub fn test_lr0_parser() {
        let table = Table::build::<0, _>(&FIXTURE_LR0_GRAMMAR).expect("cannot build table");

        let mut lexer = lexer_fixture_lr0("1 + 1 * 0".chars());
        let parser = LrParser::<AstNode<'_>>::new(
            &FIXTURE_LR0_GRAMMAR,
            &table,
            &[
                AstNode::reduce,
                AstNode::reduce,
                AstNode::reduce,
                AstNode::reduce,
                AstNode::reduce,
                AstNode::reduce,
                AstNode::reduce,
                AstNode::reduce,
                AstNode::reduce,
                AstNode::reduce,
            ],
        );

        let ast = parser.parse(&mut lexer).unwrap();
        println!("{:#?}", ast);
    }
}
