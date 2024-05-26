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

fn r1(_: &Rule, mut lhs: AstIter<Ast>) -> Result<Ast, YalpError<Error>> {
    Ok(lhs.next().unwrap())
}

fn r2(_: &Rule, mut lhs: AstIter<Ast>) -> Result<Ast, YalpError<Error>> {
    let mut set: SymbolIdentSet = lhs.next().unwrap().try_into()?;
    lhs.next();
    let ident: SymbolIdent = lhs.next().unwrap().try_into()?;

    set.0.push(ident.0);

    Ok(Ast::SymbolIdentSet(set))
}

fn r3(_: &Rule, mut lhs: AstIter<Ast>) -> Result<Ast, YalpError<Error>> {
    let ident: SymbolIdent = lhs.next().unwrap().try_into()?;

    Ok(Ast::SymbolIdentSet(SymbolIdentSet(vec![ident.0])))
}

fn r4(_: &Rule, mut lhs: AstIter<Ast>) -> Result<Ast, YalpError<Error>> {
    let chain: IdentChain = lhs.next().unwrap().try_into()?;
    Ok(Ast::SymbolIdent(SymbolIdent(chain.0)))
}

fn r5(_: &Rule, mut lhs: AstIter<Ast>) -> Result<Ast, YalpError<Error>> {
    let lit: Literal = lhs.next().unwrap().try_into()?;
    Ok(Ast::SymbolIdent(SymbolIdent(lit.to_string())))
}

fn r6(_: &Rule, mut lhs: AstIter<Ast>) -> Result<Ast, YalpError<Error>> {
    lhs.next();
    let chain: IdentChain = lhs.next().unwrap().try_into()?;

    Ok(Ast::SymbolIdent(SymbolIdent(format!("<{}>", chain.0))))
}

fn r7(_: &Rule, mut lhs: AstIter<Ast>) -> Result<Ast, YalpError<Error>> {
    let mut chain: IdentChain = lhs.next().unwrap().try_into()?;
    let mut lhs = lhs.skip(1);

    let ident: Ident = lhs.next().unwrap().try_into()?;
    chain.0.push_str(&ident.to_string());

    Ok(Ast::IdentChain(chain))
}

fn r8(_: &Rule, mut lhs: AstIter<Ast>) -> Result<Ast, YalpError<Error>> {
    let ident: Ident = lhs.next().unwrap().try_into()?;
    Ok(Ast::IdentChain(IdentChain(ident.to_string())))
}

const REDUCERS: &[RuleReducer<Ast, Error>] = &[r1, r2, r3, r4, r5, r6, r7, r8];

/// Parse a collection of symbol idents : <symbol-ident>, <symbol-ident> ...
pub fn parse_symbol_ident_set(stream: TokenStream) -> Result<SymbolIdentSet, YalpError<Error>> {
    if stream.is_empty() {
        return Ok(SymbolIdentSet::default());
    }

    let mut lexer = Lexer::new(stream);

    let table = TABLE.as_ref().unwrap();

    println!("{}", table);

    let parser = LrParser::new(&GRAMMAR, table, REDUCERS);

    let ast = parser.parse(&mut lexer)?;

    ast.try_into()
}
