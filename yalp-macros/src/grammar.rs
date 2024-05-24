use lazy_static::lazy_static;
use proc_macro2::{Group, Ident, TokenStream};
use yalp::{
    lr::LrTable,
    traits::{Ast as _, Parser as _, Token as _},
    AstIter, Grammar, LrParser, LrParserError, Rule, RuleDef, RuleReducer, Symbol, EOS, START,
};

use crate::{lexer::Lexer, parse_symbol_ident_set, Error, SymbolIdentSet, Token};

#[derive(Debug, Default)]
pub struct GrammarInput {
    terminals: SymbolIdentSet,
    non_terminals: SymbolIdentSet,
}

const GRAMMAR: Grammar<'static, 9, 4> = yalp::Grammar::new(
    [
        Symbol::start(),
        Symbol::eos(),
        Symbol::epsilon(),
        Symbol::term("<ident>"),
        Symbol::term("<group>"),
        Symbol::term(":"),
        Symbol::term(","),
        Symbol::nterm("<grammar>"),
        Symbol::nterm("<attribute>"),
    ],
    [
        RuleDef::new(START, &["<grammar>", EOS]),
        RuleDef::new("<grammar>", &["<grammar>", ",", "<attribute>"]),
        RuleDef::new("<grammar>", &["<attribute>"]),
        RuleDef::new("<attribute>", &["<ident>", ":", "<group>"]),
    ],
);

lazy_static! {
    static ref TABLE: Result<LrTable<'static, 'static>, LrParserError<'static, 'static>> =
        LrTable::build::<1, _>(&GRAMMAR);
}

#[derive(Debug)]
struct Attribute {
    name: String,
    group: Group,
}

#[derive(Debug)]
enum Ast {
    Token(Token),
    Grammar(GrammarInput),
    Attribute(Attribute),
}

impl From<GrammarInput> for Ast {
    fn from(value: GrammarInput) -> Self {
        Self::Grammar(value)
    }
}

impl From<Attribute> for Ast {
    fn from(value: Attribute) -> Self {
        Self::Attribute(value)
    }
}

impl TryFrom<Ast> for Token {
    type Error = Error;

    fn try_from(value: Ast) -> Result<Self, Self::Error> {
        match value {
            Ast::Token(tok) => Ok(tok),
            _ => Err(Error::wrong_symbol("<token>", value.symbol_id())),
        }
    }
}

impl TryFrom<Ast> for Ident {
    type Error = Error;

    fn try_from(value: Ast) -> Result<Self, Self::Error> {
        let tok: Token = value.try_into()?;
        tok.try_into()
    }
}

impl TryFrom<Ast> for Group {
    type Error = Error;

    fn try_from(value: Ast) -> Result<Self, Self::Error> {
        let tok: Token = value.try_into()?;
        tok.try_into()
    }
}

impl TryFrom<Ast> for Attribute {
    type Error = Error;

    fn try_from(value: Ast) -> Result<Self, Self::Error> {
        match value {
            Ast::Attribute(attr) => Ok(attr),
            _ => Err(Error::wrong_symbol("<attribute>", value.symbol_id())),
        }
    }
}

impl TryFrom<Ast> for GrammarInput {
    type Error = Error;

    fn try_from(value: Ast) -> Result<Self, Self::Error> {
        match value {
            Ast::Grammar(grammar) => Ok(grammar),
            _ => Err(Error::wrong_symbol("<grammar>", value.symbol_id())),
        }
    }
}

impl yalp::traits::Ast for Ast {
    fn symbol_id(&self) -> &str {
        match self {
            Self::Token(tok) => tok.symbol_id(),
            Self::Attribute(_) => "<attribute>",
            Self::Grammar(_) => "<grammar>",
        }
    }
}

impl From<Token> for Ast {
    fn from(value: Token) -> Self {
        Self::Token(value)
    }
}

/// 1. START => <grammar> EOS
fn parse_1(_: &Rule<'static, '_>, mut lhs: AstIter<Ast>) -> Result<Ast, Error> {
    Ok(lhs.next().unwrap())
}

fn merge(grammar: &mut GrammarInput, attr: Attribute) -> Result<(), Error> {
    match attr.name.as_str() {
        "terminals" => {
            grammar.terminals = parse_symbol_ident_set(attr.group.stream())?;
        }
        "non_terminals" => {
            grammar.non_terminals = parse_symbol_ident_set(attr.group.stream())?;
        }

        _ => {}
    };

    Ok(())
}

/// 2. <grammar> => <grammar> , <attribute>
fn parse_2(_: &Rule<'static, '_>, mut lhs: AstIter<Ast>) -> Result<Ast, Error> {
    let mut grammar: GrammarInput = lhs.next().unwrap().try_into()?;
    lhs.next();

    let attr: Attribute = lhs.next().unwrap().try_into()?;
    merge(&mut grammar, attr)?;

    Ok(grammar.into())
}

/// 3. <grammar> => <attribute>
fn parse_3(_: &Rule<'static, '_>, mut lhs: AstIter<Ast>) -> Result<Ast, Error> {
    let attr: Attribute = lhs.next().unwrap().try_into()?;
    let mut grammar = GrammarInput::default();
    merge(&mut grammar, attr)?;
    Ok(grammar.into())
}

/// 4. <attribute> => <ident> : <group>
fn parse_4(_: &Rule<'static, '_>, mut lhs: AstIter<Ast>) -> Result<Ast, Error> {
    let ident: Ident = lhs.next().unwrap().try_into()?;
    lhs.next();
    let group: Group = lhs.next().unwrap().try_into()?;

    Ok(Attribute {
        name: ident.to_string(),
        group,
    }
    .into())
}

const REDUCERS: &[RuleReducer<'static, Ast, Error>] = &[parse_1, parse_2, parse_3, parse_4];

pub fn parse_grammar(stream: TokenStream) -> Result<GrammarInput, Error> {
    let mut lexer = Lexer::new(stream);
    let table = TABLE
        .as_ref()
        .map_err(|err| Error::ParserError(err.clone()))?;

    println!("{}", table);

    let parser = LrParser::<Ast, _>::new(&GRAMMAR, table, &REDUCERS);

    let ast = parser.parse(&mut lexer)?;

    ast.try_into()
}
