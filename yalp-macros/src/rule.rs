use proc_macro2::{Ident, Literal};
use yalp::{
    traits::{Ast as _, Token as _},
    AstIter, RuleDef, RuleReducer, Symbol, YalpError, EOS, START,
};

use crate::{lexer::Token, Error};

const GRAMMAR: yalp::Grammar<'static, 15, 11> = yalp::Grammar::new(
    [
        Symbol::start(),
        Symbol::eos(),
        Symbol::epsilon(),
        Symbol::term("<"),
        Symbol::term(">"),
        Symbol::term(";"),
        Symbol::term("="),
        Symbol::term("-"),
        Symbol::term("<ident>"),
        Symbol::term("<lit>"),
        Symbol::nterm("<rule-set>"),
        Symbol::nterm("<rule>"),
        Symbol::nterm("<rule-rhs>"),
        Symbol::nterm("<symbol-ident>"),
        Symbol::nterm("<ident-chain>"),
    ],
    [
        RuleDef::new(START, &["<rule-set>", EOS]),
        RuleDef::new("<rule-set>", &["<rule-set>", "<rule>"]),
        RuleDef::new("<rule-set>", &["<rule>"]),
        RuleDef::new("<rule>", &["<symbol-ident>", "=", ">", "<rule-rhs>", ";"]),
        RuleDef::new("<rule-rhs>", &["<rule-rhs>", "<symbol-ident>"]),
        RuleDef::new("<rule-rhs>", &["<symbol-ident>"]),
        RuleDef::new("<symbol-ident>", &["<ident-chain>"]),
        RuleDef::new("<symbol-ident>", &["<lit>"]),
        RuleDef::new("<symbol-ident>", &["<", "<ident-chain>", ">"]),
        RuleDef::new("<ident-chain>", &["<ident-chain>", "-", "<ident>"]),
        RuleDef::new("<ident-chain>", &["<ident>"]),
    ],
);

fn r1(_: &Rule, mut rhs: AstIter<Ast>) -> Result<Ast, YalpError<Error>> {
    Ok(rhs.next().unwrap())
}

fn r2(_: &Rule, mut rhs: AstIter<Ast>) -> Result<Ast, YalpError<Error>> {
    let mut set: RuleSet = rhs.next().unwrap().try_into()?;
    let rule: Rule = rhs.next().unwrap().try_into()?;
    set.0.push(rule);
    Ok(set.into())
}

fn r3(_: &Rule, mut rhs: AstIter<Ast>) -> Result<Ast, YalpError<Error>> {
    let rule: Rule = rhs.next().unwrap().try_into()?;
    Ok(RuleSet(vec![rule]).into())
}

fn r4(_: &Rule, mut rhs: AstIter<Ast>) -> Result<Ast, YalpError<Error>> {
    let lhs: SymbolIdent = rhs.next().unwrap().try_into()?;
    rhs.next();
    rhs.next();

    let rhs: RuleRhs = rhs.next().unwrap().try_into()?;

    Ok(Rule {
        lhs: lhs.0,
        rhs: rhs.0,
    }
    .into())
}

fn r5(_: &Rule, mut iter: AstIter<Ast>) -> Result<Ast, YalpError<Error>> {
    let mut rhs: RuleRhs = iter.next().unwrap().try_into()?;
    let sym: SymbolIdent = iter.next().unwrap().try_into()?;

    rhs.0.push(sym.0);

    Ok(rhs.into())
}

fn r6(_: &Rule, mut iter: AstIter<Ast>) -> Result<Ast, YalpError<Error>> {
    let sym: SymbolIdent = iter.next().unwrap().try_into()?;
    Ok(RuleRhs(vec![sym.0]).into())
}

fn r7(_: &Rule, mut lhs: AstIter<Ast>) -> Result<Ast, YalpError<Error>> {
    let chain: IdentChain = lhs.next().unwrap().try_into()?;
    Ok(Ast::SymbolIdent(SymbolIdent(chain.0)))
}

fn r8(_: &Rule, mut lhs: AstIter<Ast>) -> Result<Ast, YalpError<Error>> {
    let lit: Literal = lhs.next().unwrap().try_into()?;
    Ok(Ast::SymbolIdent(SymbolIdent(lit.to_string())))
}

fn r9(_: &Rule, mut lhs: AstIter<Ast>) -> Result<Ast, YalpError<Error>> {
    lhs.next();
    let chain: IdentChain = lhs.next().unwrap().try_into()?;

    Ok(Ast::SymbolIdent(SymbolIdent(format!("<{}>", chain.0))))
}

fn r10(_: &Rule, mut lhs: AstIter<Ast>) -> Result<Ast, YalpError<Error>> {
    let mut chain: IdentChain = lhs.next().unwrap().try_into()?;
    let mut lhs = lhs.skip(1);

    let ident: Ident = lhs.next().unwrap().try_into()?;
    chain.0.push_str(&ident.to_string());

    Ok(Ast::IdentChain(chain))
}

fn r11(_: &Rule, mut lhs: AstIter<Ast>) -> Result<Ast, YalpError<Error>> {
    let ident: Ident = lhs.next().unwrap().try_into()?;
    Ok(Ast::IdentChain(IdentChain(ident.to_string())))
}

const REDUCERS: &[RuleReducer<Ast, Error>] = &[r1, r2, r3, r4, r5, r6, r7, r8, r9, r10, r11];

pub struct RuleSet(Vec<Rule>);

pub struct Rule {
    lhs: String,
    rhs: Vec<String>,
}

struct RuleRhs(Vec<String>);
struct SymbolIdent(String);
struct IdentChain(String);

enum Ast {
    RuleSet(RuleSet),
    Rule(Rule),
    RuleRhs(RuleRhs),
    SymbolIdent(SymbolIdent),
    IdentChain(IdentChain),
    Token(Token),
}

impl yalp::traits::Ast for Ast {
    fn symbol_id(&self) -> &str {
        match self {
            Ast::RuleSet(_) => "<rule-set>",
            Ast::Rule(_) => "<rule>",
            Ast::RuleRhs(_) => "<rule-rhs>",
            Ast::SymbolIdent(_) => "<symbol-ident>",
            Ast::IdentChain(_) => "<ident-chain>",
            Ast::Token(tok) => tok.symbol_id(),
        }
    }
}

impl From<RuleSet> for Ast {
    fn from(value: RuleSet) -> Self {
        Self::RuleSet(value)
    }
}

impl TryFrom<Ast> for RuleSet {
    type Error = YalpError<Error>;

    fn try_from(value: Ast) -> Result<Self, Self::Error> {
        match value {
            Ast::RuleSet(set) => Ok(set),
            _ => Err(YalpError::wrong_symbol("<rule-set>", value.symbol_id())),
        }
    }
}

impl From<Rule> for Ast {
    fn from(value: Rule) -> Self {
        Self::Rule(value)
    }
}

impl TryFrom<Ast> for Rule {
    type Error = YalpError<Error>;

    fn try_from(value: Ast) -> Result<Self, Self::Error> {
        match value {
            Ast::Rule(rule) => Ok(rule),
            _ => Err(YalpError::wrong_symbol("<rule>", value.symbol_id())),
        }
    }
}

impl From<RuleRhs> for Ast {
    fn from(value: RuleRhs) -> Self {
        Self::RuleRhs(value)
    }
}

impl TryFrom<Ast> for RuleRhs {
    type Error = YalpError<Error>;

    fn try_from(value: Ast) -> Result<Self, Self::Error> {
        match value {
            Ast::RuleRhs(set) => Ok(set),
            _ => Err(YalpError::wrong_symbol("<rule-rhs>", value.symbol_id())),
        }
    }
}

impl From<SymbolIdent> for Ast {
    fn from(value: SymbolIdent) -> Self {
        Self::SymbolIdent(value)
    }
}

impl TryFrom<Ast> for SymbolIdent {
    type Error = YalpError<Error>;

    fn try_from(value: Ast) -> Result<Self, Self::Error> {
        match value {
            Ast::SymbolIdent(set) => Ok(set),
            _ => Err(YalpError::wrong_symbol("<symbol-ident>", value.symbol_id())),
        }
    }
}

impl From<IdentChain> for Ast {
    fn from(value: IdentChain) -> Self {
        Self::IdentChain(value)
    }
}

impl TryFrom<Ast> for IdentChain {
    type Error = YalpError<Error>;

    fn try_from(value: Ast) -> Result<Self, Self::Error> {
        match value {
            Ast::IdentChain(set) => Ok(set),
            _ => Err(YalpError::wrong_symbol("<ident-chain>", value.symbol_id())),
        }
    }
}

impl From<Token> for Ast {
    fn from(value: Token) -> Self {
        Self::Token(value)
    }
}

impl TryFrom<Ast> for Token {
    type Error = YalpError<Error>;

    fn try_from(value: Ast) -> Result<Self, Self::Error> {
        match value {
            Ast::Token(set) => Ok(set),
            _ => Err(YalpError::wrong_symbol("<token>", value.symbol_id())),
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
