use lazy_static::lazy_static;
use proc_macro2::{Ident, Literal, TokenStream};

use crate::{Error, Lexer, Token};
use yalp::{
    lr::LrTable,
    traits::{Ast as _, Parser as _, Token as _},
    AstIter, LrParser, LrParserError, Rule, RuleReducer, YalpError, EOS, START,
};

#[derive(Debug, Default)]
pub struct SymbolIdentSet(Vec<String>);

const GRAMMAR: yalp::Grammar<'static, 12, 8> = yalp::Grammar::new(
    [
        yalp::Symbol::start(),
        yalp::Symbol::eos(),
        yalp::Symbol::epsilon(),
        yalp::Symbol::term("<"),
        yalp::Symbol::term(">"),
        yalp::Symbol::term(","),
        yalp::Symbol::term("-"),
        yalp::Symbol::term("<ident>"),
        yalp::Symbol::term("<lit>"),
        yalp::Symbol::nterm("<symbol-ident-set>"),
        yalp::Symbol::nterm("<symbol-ident>"),
        yalp::Symbol::nterm("<ident-chain>"),
    ],
    [
        yalp::RuleDef::new(START, &["<symbol-ident-set>", EOS]),
        yalp::RuleDef::new(
            "<symbol-ident-set>",
            &["<symbol-ident-set>", ",", "<symbol-ident>"],
        ),
        yalp::RuleDef::new("<symbol-ident-set>", &["<symbol-ident>"]),
        yalp::RuleDef::new("<symbol-ident>", &["<ident-chain>"]),
        yalp::RuleDef::new("<symbol-ident>", &["<lit>"]),
        yalp::RuleDef::new("<symbol-ident>", &["<", "<ident-chain>", ">"]),
        yalp::RuleDef::new("<ident-chain>", &["<ident-chain>", "-", "<ident>"]),
        yalp::RuleDef::new("<ident-chain>", &["<ident>"]),
    ],
);

lazy_static! {
    static ref TABLE: Result<LrTable<'static, 'static>, LrParserError> =
        LrTable::build::<0, _>(&GRAMMAR);
}

struct SymbolIdent(String);

struct IdentChain(String);

enum Ast {
    Token(Token),
    IdentChain(IdentChain),
    SymbolIdent(SymbolIdent),
    SymbolIdentSet(SymbolIdentSet),
}

impl TryFrom<Ast> for SymbolIdentSet {
    type Error = YalpError<Error>;

    fn try_from(value: Ast) -> Result<Self, Self::Error> {
        match value {
            Ast::SymbolIdentSet(set) => Ok(set),
            _ => Err(Self::Error::wrong_symbol(
                "<symbol-ident-set>",
                value.symbol_id(),
            )),
        }
    }
}

impl TryFrom<Ast> for SymbolIdent {
    type Error = YalpError<Error>;

    fn try_from(value: Ast) -> Result<Self, Self::Error> {
        match value {
            Ast::SymbolIdent(chain) => Ok(chain),
            _ => Err(Self::Error::wrong_symbol(
                "<symbol-ident>",
                value.symbol_id(),
            )),
        }
    }
}

impl TryFrom<Ast> for IdentChain {
    type Error = YalpError<Error>;

    fn try_from(value: Ast) -> Result<Self, Self::Error> {
        match value {
            Ast::IdentChain(chain) => Ok(chain),
            _ => Err(Self::Error::wrong_symbol(
                "<ident-chain>",
                value.symbol_id(),
            )),
        }
    }
}

impl TryFrom<Ast> for Token {
    type Error = YalpError<Error>;

    fn try_from(value: Ast) -> Result<Self, Self::Error> {
        match value {
            Ast::Token(tok) => Ok(tok),
            _ => Err(Self::Error::wrong_symbol(
                "<ident-chain>",
                value.symbol_id(),
            )),
        }
    }
}

impl TryFrom<Ast> for Ident {
    type Error = YalpError<Error>;

    fn try_from(value: Ast) -> Result<Self, Self::Error> {
        let tok: Token = value.try_into()?;
        tok.try_into()
    }
}

impl TryFrom<Ast> for Literal {
    type Error = YalpError<Error>;

    fn try_from(value: Ast) -> Result<Self, Self::Error> {
        let tok: Token = value.try_into()?;
        tok.try_into()
    }
}

impl yalp::traits::Ast for Ast {
    fn symbol_id(&self) -> &str {
        match self {
            Ast::Token(tok) => tok.symbol_id(),
            Ast::IdentChain(_) => "<ident-chain>",
            Ast::SymbolIdent(_) => "<symbol-ident>",
            Ast::SymbolIdentSet(_) => "<symbol-ident-set>",
        }
    }
}

impl From<Token> for Ast {
    fn from(value: Token) -> Self {
        Self::Token(value)
    }
}

///////////////////
// Rule reducers //
///////////////////

/// 1. START => <symbol-set-1> EOS
fn parse_1(_: &Rule<'static>, mut lhs: AstIter<Ast>) -> Result<Ast, YalpError<Error>> {
    Ok(lhs.next().unwrap())
}

/// 2. <symbol-ident-set> => <symbol-ident-set>" , <symbol-ident>
fn parse_2(_: &Rule<'static>, mut lhs: AstIter<Ast>) -> Result<Ast, YalpError<Error>> {
    let mut set: SymbolIdentSet = lhs.next().unwrap().try_into()?;
    lhs.next();
    let ident: SymbolIdent = lhs.next().unwrap().try_into()?;

    set.0.push(ident.0);

    Ok(Ast::SymbolIdentSet(set))
}

/// 3. <symbol-ident-set> =>  <symbol-ident>
fn parse_3(_: &Rule<'static>, mut lhs: AstIter<Ast>) -> Result<Ast, YalpError<Error>> {
    let ident: SymbolIdent = lhs.next().unwrap().try_into()?;

    Ok(Ast::SymbolIdentSet(SymbolIdentSet(vec![ident.0])))
}

/// 3. <symbol-ident> => <ident-chain>
fn parse_4(_: &Rule<'static>, mut lhs: AstIter<Ast>) -> Result<Ast, YalpError<Error>> {
    let chain: IdentChain = lhs.next().unwrap().try_into()?;
    Ok(Ast::SymbolIdent(SymbolIdent(chain.0)))
}

/// 4. <symbol-ident> <lit>
fn parse_5(_: &Rule<'static>, mut lhs: AstIter<Ast>) -> Result<Ast, YalpError<Error>> {
    let lit: Literal = lhs.next().unwrap().try_into()?;
    Ok(Ast::SymbolIdent(SymbolIdent(lit.to_string())))
}

/// 5. <symbol-ident> => < <ident-chain> >
fn parse_6(_: &Rule<'static>, mut lhs: AstIter<Ast>) -> Result<Ast, YalpError<Error>> {
    lhs.next();
    let chain: IdentChain = lhs.next().unwrap().try_into()?;

    Ok(Ast::SymbolIdent(SymbolIdent(format!("<{}>", chain.0))))
}

/// 6. <ident-chain> => <ident-chain> - <ident>
fn parse_7(_: &Rule<'static>, mut lhs: AstIter<Ast>) -> Result<Ast, YalpError<Error>> {
    let mut chain: IdentChain = lhs.next().unwrap().try_into()?;
    let mut lhs = lhs.skip(1);

    let ident: Ident = lhs.next().unwrap().try_into()?;
    chain.0.push_str(&ident.to_string());

    Ok(Ast::IdentChain(chain))
}

/// 7. <ident-chain> => <ident>
fn parse_8(_: &Rule<'static>, mut lhs: AstIter<Ast>) -> Result<Ast, YalpError<Error>> {
    let ident: Ident = lhs.next().unwrap().try_into()?;
    Ok(Ast::IdentChain(IdentChain(ident.to_string())))
}

const REDUCERS: &[RuleReducer<'static, Ast, Error>] = &[
    parse_1, parse_2, parse_3, parse_4, parse_5, parse_6, parse_7, parse_8,
];

/// Parse a collection of symbol idents : <symbol-ident>, <symbol-ident> ...
pub fn parse_symbol_ident_set(stream: TokenStream) -> Result<SymbolIdentSet, YalpError<Error>> {
    if stream.is_empty() {
        return Ok(SymbolIdentSet::default());
    }

    let mut lexer = Lexer::new(stream);

    let table = TABLE.as_ref().unwrap();

    println!("{}", table);

    let parser = LrParser::new(&GRAMMAR, table, &REDUCERS);

    let ast = parser.parse(&mut lexer)?;

    ast.try_into()
}
