use std::fmt::Debug;
use std::marker::{PhantomData, PhantomPinned};

use crate::grammar::traits::Grammar;
use crate::token::traits::Token;
use crate::traits::SymbolSlice as _;
use crate::{
    lexer::{traits::Lexer, LexerError},
    parser::{traits::Ast, traits::Parser},
    ItemSetId, RuleId, RuleReducer, RuleSet,
};
use crate::{AstIter, OwnedSymbol, Rule, YalpError};

mod action;
mod codegen;
mod graph;
mod table;
mod transition;

pub use action::*;
use graph::*;
pub use table::*;
use transition::*;

#[derive(Debug)]
pub enum LrParserError {
    MissingRule(RuleId),
    MissingAction(ItemSetId, OwnedSymbol),
    MissingGoto(ItemSetId, OwnedSymbol),
    MissingState(ItemSetId),
    LexerError(LexerError),
    UnknownSymbol(String),
    UnexpectedSymbol {
        expected: String,
        got: String,
    },
    UnsupportedLrRank,
    ShiftReduceConflict {
        state: ItemSetId,
        symbol: OwnedSymbol,
        conflict: [Action; 2],
    },
    Custom(String),
}

impl From<LexerError> for LrParserError {
    fn from(value: LexerError) -> Self {
        Self::LexerError(value)
    }
}

impl std::fmt::Display for LrParserError {
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
            LrParserError::LexerError(error) => write!(f, "{error}"),
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
            LrParserError::UnknownSymbol(symbol_id) => write!(f, "unknown symbol {symbol_id}"),
            LrParserError::UnexpectedSymbol { expected, got } => {
                write!(f, "unexpected symbol {}, expecting {}", got, expected)
            }
            LrParserError::Custom(err) => write!(f, "{err}"),
        }
    }
}

pub type LrResult<T> = Result<T, LrParserError>;

pub struct LrParser<'sid, 'sym, 'table, 'reducers, Node, Table, Reducer, Error>
where
    Node: Ast,
    Table: self::traits::LrTable,
    Reducer: Fn(&Rule, AstIter<Node>) -> Result<Node, YalpError<Error>>,
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
    Reducer: Fn(&Rule, AstIter<Node>) -> Result<Node, YalpError<Error>>,
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

impl<'sid, 'sym, 'table, 'reducers, Node, Table, Reducer, Error> Parser
    for LrParser<'sid, 'sym, 'table, 'reducers, Node, Table, Reducer, Error>
where
    Node: Ast,
    Table: self::traits::LrTable,
    Reducer: Fn(&Rule, AstIter<Node>) -> Result<Node, YalpError<Error>>,
{
    type Ast = Node;
    type Error = YalpError<Error>;

    fn parse<L: Lexer>(&self, lexer: &mut L) -> Result<Self::Ast, Self::Error>
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
                        .get_symbol_by_id(tok.symbol_id())
                        .ok_or_else(|| LrParserError::UnknownSymbol(tok.symbol_id().to_string()))
                        .map_err(Self::Error::from)?,
                    Some(tok),
                ),
                Some(Err(err)) => return Err(LrParserError::LexerError(err.clone()).into()),
            };

            let action = self
                .table
                .action(state, &symbol)
                .ok_or(LrParserError::MissingAction(state, symbol.to_owned()))?;

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
                                    Err(LrParserError::UnexpectedSymbol {
                                        expected: expected_symbol.id.to_string(),
                                        got: node.symbol_id().to_string(),
                                    })
                                } else {
                                    Ok(())
                                }
                            })?;

                        states.truncate(states.len().saturating_sub(consume));
                        state = states.last().copied().unwrap();

                        let goto = self
                            .table
                            .goto(state, &rule.lhs)
                            .ok_or(LrParserError::MissingGoto(state, rule.lhs.to_owned()))?;

                        states.push(goto);

                        let reducer = self.reducers.get(*rule_id).unwrap();
                        reducer(rule, drained)
                    }?;

                    if ast.symbol_id() != rule.lhs.id {
                        return Err(LrParserError::UnexpectedSymbol {
                            expected: rule.lhs.id.to_owned(),
                            got: ast.symbol_id().to_string(),
                        }
                        .into());
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
        ast::{ast_reduce, AstNode},
        fixtures::{FIXTURE_LR0_GRAMMAR, FIXTURE_LR1_GRAMMAR},
        lexer::fixtures::{lexer_fixture_lr0, lexer_fixture_lr1},
        traits::Parser as _,
    };

    use super::{LrParser, LrTable};

    #[test]
    pub fn test_lr0_grammar_table_building() {
        let table = LrTable::build::<0, _>(&FIXTURE_LR0_GRAMMAR).expect("cannot build table");
        println!("{}", table);
    }

    #[test]
    pub fn test_lr1_grammar_table_building() {
        let table = LrTable::build::<1, _>(&FIXTURE_LR1_GRAMMAR).expect("cannot build table");
        println!("{}", table);
    }

    #[test]
    pub fn test_lr0_parser() {
        let table = LrTable::build::<0, _>(&FIXTURE_LR0_GRAMMAR).expect("cannot build table");

        let mut lexer = lexer_fixture_lr0("1 + 1 * 0 * 1 * 1".chars());

        let parser = LrParser::new(
            &FIXTURE_LR0_GRAMMAR,
            &table,
            &[
                ast_reduce, ast_reduce, ast_reduce, ast_reduce, ast_reduce, ast_reduce,
            ],
        );

        let ast = parser.parse(&mut lexer).unwrap();
        println!("{:#?}", ast);
    }

    #[test]
    pub fn test_lr1_parser() {
        let table = LrTable::build::<1, _>(&FIXTURE_LR1_GRAMMAR).expect("cannot build table");

        let mut lexer = lexer_fixture_lr1("n + n".chars());
        let parser = LrParser::new(
            &FIXTURE_LR1_GRAMMAR,
            &table,
            &[
                ast_reduce, ast_reduce, ast_reduce, ast_reduce, ast_reduce, ast_reduce,
            ],
        );

        let ast = parser.parse(&mut lexer).unwrap();
        println!("{:#?}", ast);
    }
}
