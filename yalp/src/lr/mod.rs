use std::fmt::Debug;

use crate::grammar::traits::Grammar;
use crate::token::traits::Token;
use crate::traits::IntoRef;
use crate::traits::SymbolSliceable as _;
use crate::{
    lexer::{traits::Lexer, LexerError},
    parser::{traits::Ast, traits::Parser},
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
    table: &'table LrTable<'sid, 'sym>,
    reducers: &'reducers [RuleReducer<'sid, Node>],
}

impl<'sid, 'g, 'table, 'reducers, Node> LrParser<'sid, 'g, 'table, 'reducers, Node>
where
    Node: Ast,
{
    pub fn new<G>(
        grammar: &'g G,
        table: &'table LrTable<'sid, 'g>,
        reducers: &'reducers [RuleReducer<'sid, Node>],
    ) -> Self
    where
        G: Grammar<'sid, 'g>,
        &'g G: IntoRef<'g, [Symbol<'sid>]>,
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
        let mut states: Vec<ItemSetId> = vec![0];
        let mut stack: Vec<Node> = Vec::default();

        println!("{}", self.table);

        let mut cursor = lexer.next();

        loop {
            let mut state = states.last().copied().unwrap();

            let (symbol, tok) = match &cursor {
                None => (self.rules.eos(), None),
                Some(Ok(tok)) => (
                    self.rules
                        .get_symbol_by_id(tok.symbol_id())
                        .ok_or_else(|| LrParserError::UnknownSymbol(tok.symbol_id().to_string()))?,
                    Some(tok),
                ),
                Some(Err(err)) => return Err(LrParserError::LexerError(err.clone())),
            };

            let row = self
                .table
                .get(state)
                .ok_or(LrParserError::MissingState(state))?;

            let action = row
                .action(symbol)
                .ok_or(LrParserError::MissingAction(state, symbol))?;

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
                    let rule = self.rules.get(*rule_id);
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
                                        expected: expected_symbol.id,
                                        got: node.symbol_id().to_string(),
                                    })
                                } else {
                                    Ok(())
                                }
                            })?;

                        states.truncate(states.len().saturating_sub(consume));
                        state = states.last().copied().unwrap();

                        let row = self
                            .table
                            .get(state)
                            .ok_or(LrParserError::MissingState(state))?;

                        let goto = row
                            .goto(rule.lhs)
                            .ok_or(LrParserError::MissingGoto(state, rule.lhs))?;

                        states.push(goto);

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
        ast::AstNode,
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
            ],
        );

        let ast = parser.parse(&mut lexer).unwrap();
        println!("{:#?}", ast);
    }

    #[test]
    pub fn test_lr1_parser() {
        let table = LrTable::build::<1, _>(&FIXTURE_LR1_GRAMMAR).expect("cannot build table");

        let mut lexer = lexer_fixture_lr1("n + n".chars());
        let parser = LrParser::<AstNode<'_>>::new(
            &FIXTURE_LR1_GRAMMAR,
            &table,
            &[
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
