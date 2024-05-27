use lazy_static::lazy_static;
use proc_macro2::{Ident, Literal, TokenStream};
use quote::quote;

use crate::{Error, Lexer, Token};
use yalp_core::{
    lr::LrTable, traits::{Ast as _, Parser as _, Token as _}, ConstRuleReducer, LrParser, Rule, RuleReducer, RuleRhs, YalpError, YalpResult, EOS, START
};

#[derive(Debug, Default)]
pub struct SymbolIdentSet(pub Vec<String>);

const GRAMMAR: yalp_core::ConstGrammar<'static, 12, 8> = yalp_core::ConstGrammar::new(
    [
        yalp_core::Symbol::start(),
        yalp_core::Symbol::eos(),
        yalp_core::Symbol::epsilon(),
        yalp_core::Symbol::term("<"),
        yalp_core::Symbol::term(">"),
        yalp_core::Symbol::term(","),
        yalp_core::Symbol::term("-"),
        yalp_core::Symbol::term("<ident>"),
        yalp_core::Symbol::term("<lit>"),
        yalp_core::Symbol::nterm("<symbol-ident-set>"),
        yalp_core::Symbol::nterm("<symbol-ident>"),
        yalp_core::Symbol::nterm("<ident-chain>"),
    ],
    [
        yalp_core::RuleDef::new(START, &["<symbol-ident-set>", EOS]),
        yalp_core::RuleDef::new(
            "<symbol-ident-set>",
            &["<symbol-ident-set>", ",", "<symbol-ident>"],
        ),
        yalp_core::RuleDef::new("<symbol-ident-set>", &["<symbol-ident>"]),
        yalp_core::RuleDef::new("<symbol-ident>", &["<ident-chain>"]),
        yalp_core::RuleDef::new("<symbol-ident>", &["<lit>"]),
        yalp_core::RuleDef::new("<symbol-ident>", &["<", "<ident-chain>", ">"]),
        yalp_core::RuleDef::new("<ident-chain>", &["<ident-chain>", "-", "<ident>"]),
        yalp_core::RuleDef::new("<ident-chain>", &["<ident>"]),
    ],
);

lazy_static! {
    static ref TABLE: YalpResult<LrTable<'static, 'static>, Error> =
        LrTable::build::<0, _, _>(&GRAMMAR);
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
            _ => Err(yalp_core::ErrorKind::unexpected_symbol(
                "<symbol-ident-set>",
                [value.symbol_id()],
            ).into()),
        }
    }
}

impl TryFrom<Ast> for SymbolIdent {
    type Error = YalpError<Error>;

    fn try_from(value: Ast) -> Result<Self, Self::Error> {
        match value {
            Ast::SymbolIdent(chain) => Ok(chain),
            _ => Err(yalp_core::ErrorKind::unexpected_symbol(
                "<symbol-ident>",
                [value.symbol_id()],
            ).into()),
        }
    }
}

impl TryFrom<Ast> for IdentChain {
    type Error = YalpError<Error>;

    fn try_from(value: Ast) -> Result<Self, Self::Error> {
        match value {
            Ast::IdentChain(chain) => Ok(chain),
            _ => Err(yalp_core::ErrorKind::unexpected_symbol(
                "<ident-chain>",
                [value.symbol_id()],
            ).into()),
        }
    }
}

impl TryFrom<Ast> for Token {
    type Error = YalpError<Error>;

    fn try_from(value: Ast) -> Result<Self, Self::Error> {
        match value {
            Ast::Token(tok) => Ok(tok),
            _ => Err(yalp_core::ErrorKind::unexpected_symbol(
                "<ident-chain>",
                [value.symbol_id()],
            ).into()),
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

impl yalp_core::traits::Ast for Ast {
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

fn r1(_: &Rule, mut lhs: RuleRhs<Ast>) -> Result<Ast, YalpError<Error>> {
    Ok(lhs.next().unwrap())
}

fn r2(_: &Rule, mut lhs: RuleRhs<Ast>) -> Result<Ast, YalpError<Error>> {
    let mut set: SymbolIdentSet = lhs.next().unwrap().try_into()?;
    lhs.next();
    let ident: SymbolIdent = lhs.next().unwrap().try_into()?;

    set.0.push(ident.0);

    Ok(Ast::SymbolIdentSet(set))
}

fn r3(_: &Rule, mut lhs: RuleRhs<Ast>) -> Result<Ast, YalpError<Error>> {
    let ident: SymbolIdent = lhs.next().unwrap().try_into()?;

    Ok(Ast::SymbolIdentSet(SymbolIdentSet(vec![ident.0])))
}

fn r4(_: &Rule, mut lhs: RuleRhs<Ast>) -> Result<Ast, YalpError<Error>> {
    let chain: IdentChain = lhs.next().unwrap().try_into()?;
    Ok(Ast::SymbolIdent(SymbolIdent(chain.0)))
}

fn r5(_: &Rule, mut lhs: RuleRhs<Ast>) -> Result<Ast, YalpError<Error>> {
    let lit: Literal = lhs.next().unwrap().try_into()?;
    Ok(Ast::SymbolIdent(SymbolIdent(lit.to_string())))
}

fn r6(_: &Rule, mut lhs: RuleRhs<Ast>) -> Result<Ast, YalpError<Error>> {
    lhs.next();
    let chain: IdentChain = lhs.next().unwrap().try_into()?;

    Ok(Ast::SymbolIdent(SymbolIdent(format!("<{}>", chain.0))))
}

fn r7(_: &Rule, mut lhs: RuleRhs<Ast>) -> Result<Ast, YalpError<Error>> {
    let mut chain: IdentChain = lhs.next().unwrap().try_into()?;
    let mut lhs = lhs.skip(1);

    let ident: Ident = lhs.next().unwrap().try_into()?;
    chain.0.push('-');
    chain.0.push_str(&ident.to_string());

    Ok(Ast::IdentChain(chain))
}

fn r8(_: &Rule, mut lhs: RuleRhs<Ast>) -> Result<Ast, YalpError<Error>> {
    let ident: Ident = lhs.next().unwrap().try_into()?;
    Ok(Ast::IdentChain(IdentChain(ident.to_string())))
}

const REDUCERS: &[ConstRuleReducer<Ast, Error>] = &[
    RuleReducer::new(r1), 
    RuleReducer::new(r2), 
    RuleReducer::new(r3), 
    RuleReducer::new(r4), 
    RuleReducer::new(r5), 
    RuleReducer::new(r6), 
    RuleReducer::new(r7), 
    RuleReducer::new(r8)
];

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
