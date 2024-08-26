use lazy_static::lazy_static;
use quote::quote;
use proc_macro2::{Group, Ident, TokenStream};
use yalp_core::{
    lr::LrTable, traits::{Ast as _, Parser as _, Token as _}, ConstRuleReducer, ConstGrammar, LrParser, Rule, RuleDef, RuleReducer, RuleRhs, Symbol, YalpError, YalpResult, EOS, START
};

use crate::{lexer::Lexer, parse_symbol_ident_set, rule::parse_rule_set, Error, RuleSet, SymbolIdentSet, Token};

#[derive(Debug, Default)]
pub struct GrammarInput {
    terminals: SymbolIdentSet,
    non_terminals: SymbolIdentSet,
    rules: RuleSet
}

impl GrammarInput {
    pub fn into_token_stream(&self) -> TokenStream {
        let symbols = [
            quote!{yalp::Symbol::start()},
            quote!{yalp::Symbol::eos()},
            quote!{yalp::Symbol::epsilon()}
        ].into_iter()
        .chain(self.terminals.0.iter().map(|s| quote!{yalp::Symbol::term(#s)}))
        .chain(self.non_terminals.0.iter().map(|s| quote!{yalp::Symbol::nterm(#s)}));

        let rules = self.rules.into_token_stream();

        quote! {
            yalp::ConstGrammar::new([#(#symbols),*], #rules)
        }.into()
    }
}

const GRAMMAR: ConstGrammar<'static, 9, 4> = yalp_core::ConstGrammar::new(
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
    static ref TABLE: YalpResult<LrTable<'static, 'static>, Error> =
        LrTable::build::<1, _, _>(&GRAMMAR);
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
    type Error = YalpError<Error>;

    fn try_from(value: Ast) -> Result<Self, Self::Error> {
        match value {
            Ast::Token(tok) => Ok(tok),
            _ => Err(yalp_core::ErrorKind::unexpected_symbol("<token>", [value.symbol_id()]).into()),
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

impl TryFrom<Ast> for Group {
    type Error = YalpError<Error>;

    fn try_from(value: Ast) -> Result<Self, Self::Error> {
        let tok: Token = value.try_into()?;
        tok.try_into()
    }
}

impl TryFrom<Ast> for Attribute {
    type Error = YalpError<Error>;

    fn try_from(value: Ast) -> Result<Self, Self::Error> {
        match value {
            Ast::Attribute(attr) => Ok(attr),
            _ => Err(yalp_core::ErrorKind::unexpected_symbol("<attribute>", [value.symbol_id()]).into()),
        }
    }
}

impl TryFrom<Ast> for GrammarInput {
    type Error = YalpError<Error>;

    fn try_from(value: Ast) -> Result<Self, Self::Error> {
        match value {
            Ast::Grammar(grammar) => Ok(grammar),
            _ => Err(yalp_core::ErrorKind::unexpected_symbol("<grammar>", [value.symbol_id()]).into()),
        }
    }
}

impl yalp_core::traits::Ast for Ast {
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
fn r1(_: &Rule, mut lhs: RuleRhs<Ast>) -> Result<Ast, YalpError<Error>> {
    Ok(lhs.next().unwrap())
}

fn merge(grammar: &mut GrammarInput, attr: Attribute) -> Result<(), YalpError<Error>> {
    match attr.name.as_str() {
        "terminals" => {
            grammar.terminals = parse_symbol_ident_set(attr.group.stream())?;
        }
        "non_terminals" => {
            grammar.non_terminals = parse_symbol_ident_set(attr.group.stream())?;
        }
        "rules" => {
            grammar.rules = parse_rule_set(attr.group.stream())?
        }
        _ => {}
    };

    Ok(())
}

/// 2. <grammar> => <grammar> , <attribute>
fn r2(_: &Rule, mut lhs: RuleRhs<Ast>) -> Result<Ast, YalpError<Error>> {
    let mut grammar: GrammarInput = lhs.next().unwrap().try_into()?;
    lhs.next();

    let attr: Attribute = lhs.next().unwrap().try_into()?;
    merge(&mut grammar, attr)?;

    Ok(grammar.into())
}

/// 3. <grammar> => <attribute>
fn r3(_: &Rule, mut lhs: RuleRhs<Ast>) -> Result<Ast, YalpError<Error>> {
    let attr: Attribute = lhs.next().unwrap().try_into()?;
    let mut grammar = GrammarInput::default();
    merge(&mut grammar, attr)?;
    Ok(grammar.into())
}

/// 4. <attribute> => <ident> : <group>
fn r4(_: &Rule, mut lhs: RuleRhs<Ast>) -> Result<Ast, YalpError<Error>> {
    let ident: Ident = lhs.next().unwrap().try_into()?;
    lhs.next();
    let group: Group = lhs.next().unwrap().try_into()?;

    Ok(Attribute {
        name: ident.to_string(),
        group,
    }
    .into())
}

const REDUCERS: &[ConstRuleReducer<Ast, Error>] = &[
    RuleReducer::new(r1), 
    RuleReducer::new(r2), 
    RuleReducer::new(r3), 
    RuleReducer::new(r4)
];

pub fn parse_grammar(stream: TokenStream) -> Result<GrammarInput, YalpError<Error>> {
    let mut lexer = Lexer::new(stream);
    let table = TABLE.as_ref().unwrap();

    println!("{}", table);

    let parser = LrParser::new(&GRAMMAR, table, REDUCERS);

    let ast = parser.parse(&mut lexer)?;

    ast.try_into()
}
