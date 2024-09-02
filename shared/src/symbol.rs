use std::{
    borrow::Cow,
    ops::{Deref, Index},
};

use crate::{prelude::IterSymbolIdentifiers, syntax::Syntax};

pub struct SymbolSet<'syntax>(Vec<Symbol<'syntax>>);

impl<'syntax, Def> From<&Syntax<'syntax, Def>> for SymbolSet<'syntax>
where
    Def: Clone + IterSymbolIdentifiers<'syntax>,
{
    fn from(syntax: &Syntax<'syntax, Def>) -> Self {
        Self(
            syntax
                .iter_symbol_identifiers()
                .map(|id| {
                    if syntax.is_terminal(&id) {
                        Symbol::Terminal(Terminal(id))
                    } else {
                        Symbol::NonTerminal(NonTerminal {
                            id,
                            is_start: false,
                        })
                    }
                })
                .collect(),
        )
    }
}

impl<'syntax, SymId> Index<SymId> for SymbolSet<'syntax>
where
    SymId: AsRef<str>,
{
    type Output = Symbol<'syntax>;

    fn index(&self, index: SymId) -> &Self::Output {
        self.0.iter().find(move |sym| sym.is(&index)).unwrap()
    }
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct SymbolId<'a>(Cow<'a, str>);

impl AsRef<str> for SymbolId<'_> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for SymbolId<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SymbolId<'_> {
    pub fn is<SymId: AsRef<str>>(&self, id: SymId) -> bool {
        self.deref() == id.as_ref()
    }
}
pub type StaticSymbolId = SymbolId<'static>;

impl From<String> for SymbolId<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<&'a str> for SymbolId<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Terminal<'syntax>(SymbolId<'syntax>);

impl<'syntax> AsRef<SymbolId<'syntax>> for Terminal<'syntax> {
    fn as_ref(&self) -> &SymbolId<'syntax> {
        &self.0
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct NonTerminal<'syntax> {
    id: SymbolId<'syntax>,
    is_start: bool,
}

impl<'syntax> AsRef<SymbolId<'syntax>> for NonTerminal<'syntax> {
    fn as_ref(&self) -> &SymbolId<'syntax> {
        &self.id
    }
}

/// Symbol definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Symbol<'syntax> {
    Terminal(Terminal<'syntax>),
    NonTerminal(NonTerminal<'syntax>),
    EOS,
}

impl Symbol<'_> {
    pub fn is_eos(&self) -> bool {
        matches!(self, Self::EOS)
    }

    pub fn is_start(&self) -> bool {
        matches!(self, Self::NonTerminal(sym) if sym.is_start)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Terminal(_)) || matches!(self, Self::EOS)
    }

    pub fn is_non_terminal(&self) -> bool {
        matches!(self, Self::NonTerminal(_))
    }

    pub fn is<SymId: AsRef<str>>(&self, id: SymId) -> bool {
        match self {
            Self::Terminal(sym) => sym.as_ref().is(id),
            Self::NonTerminal(sym) => sym.as_ref().is(id),
            Self::EOS => false,
        }
    }
}

pub struct SymbolFragment(String);

impl SymbolFragment {
    pub fn into_string(self) -> String {
        self.0
    }
    pub fn is_parsable(input: &syn::parse::ParseStream) -> bool {
        use syn::Token;

        input.peek(Token![as])
            || input.peek(Token![break])
            || input.peek(Token![const])
            || input.peek(Token![continue])
            || input.peek(Token![crate])
            || input.peek(Token![else])
            || input.peek(Token![enum])
            || input.peek(Token![extern])
            || input.peek(syn::LitBool)
            || input.peek(Token![fn])
            || input.peek(Token![for])
            || input.peek(Token![if])
            || input.peek(Token![impl])
            || input.peek(Token![in])
            || input.peek(Token![let])
            || input.peek(Token![loop])
            || input.peek(Token![match])
            || input.peek(Token![mod])
            || input.peek(Token![move])
            || input.peek(Token![mut])
            || input.peek(Token![pub])
            || input.peek(Token![ref])
            || input.peek(Token![return])
            || input.peek(Token![self])
            || input.peek(Token![Self])
            || input.peek(Token![static])
            || input.peek(Token![struct])
            || input.peek(Token![super])
            || input.peek(Token![trait])
            || input.peek(Token![type])
            || input.peek(Token![unsafe])
            || input.peek(Token![use])
            || input.peek(Token![where])
            || input.peek(Token![while])
            || input.peek(Token![-])
            || input.peek(syn::Ident)
            || input.peek(syn::LitInt)
    }
}

impl syn::parse::Parse for SymbolFragment {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::{LitBool, LitInt, Token};

        if input.peek(Token![as]) {
            input.parse::<Token![as]>()?;
            Ok(Self("as".to_owned()))
        } else if input.peek(Token![break]) {
            input.parse::<Token![break]>()?;
            Ok(Self("break".to_owned()))
        } else if input.peek(Token![const]) {
            input.parse::<Token![const]>()?;
            Ok(Self("const".to_owned()))
        } else if input.peek(Token![continue]) {
            input.parse::<Token![continue]>()?;
            Ok(Self("continue".to_owned()))
        } else if input.peek(Token![crate]) {
            input.parse::<Token![crate]>()?;
            Ok(Self("crate".to_owned()))
        } else if input.peek(Token![else]) {
            input.parse::<Token![else]>()?;
            Ok(Self("else".to_owned()))
        } else if input.peek(Token![enum]) {
            input.parse::<Token![enum]>()?;
            Ok(Self("enum".to_owned()))
        } else if input.peek(Token![extern]) {
            input.parse::<Token![extern]>()?;
            Ok(Self("extern".to_owned()))
        } else if input.peek(LitBool) {
            let lit = input.parse::<LitBool>()?;
            let val = lit
                .value()
                .then(|| String::from("true"))
                .unwrap_or(String::from("false"));
            Ok(Self(val))
        } else if input.peek(LitInt) {
            let lit = input.parse::<LitInt>()?;
            let val = lit.to_string();
            Ok(Self(val))
        } else if input.peek(Token![fn]) {
            input.parse::<Token![fn]>()?;
            Ok(Self("fn".to_owned()))
        } else if input.peek(Token![for]) {
            input.parse::<Token![for]>()?;
            Ok(Self("for".to_owned()))
        } else if input.peek(Token![if]) {
            input.parse::<Token![if]>()?;
            Ok(Self("if".to_owned()))
        } else if input.peek(Token![impl]) {
            input.parse::<Token![impl]>()?;
            Ok(Self("impl".to_owned()))
        } else if input.peek(Token![in]) {
            input.parse::<Token![in]>()?;
            Ok(Self("in".to_owned()))
        } else if input.peek(Token![let]) {
            input.parse::<Token![let]>()?;
            Ok(Self("let".to_owned()))
        } else if input.peek(Token![loop]) {
            input.parse::<Token![loop]>()?;
            Ok(Self("loop".to_owned()))
        } else if input.peek(Token![match]) {
            input.parse::<Token![match]>()?;
            Ok(Self("match".to_owned()))
        } else if input.peek(Token![mod]) {
            input.parse::<Token![mod]>()?;
            Ok(Self("mod".to_owned()))
        } else if input.peek(Token![move]) {
            input.parse::<Token![move]>()?;
            Ok(Self("move".to_owned()))
        } else if input.peek(Token![mut]) {
            input.parse::<Token![mut]>()?;
            Ok(Self("mut".to_owned()))
        } else if input.peek(Token![pub]) {
            input.parse::<Token![pub]>()?;
            Ok(Self("pub".to_owned()))
        } else if input.peek(Token![ref]) {
            input.parse::<Token![ref]>()?;
            Ok(Self("ref".to_owned()))
        } else if input.peek(Token![return]) {
            input.parse::<Token![return]>()?;
            Ok(Self("return".to_owned()))
        } else if input.peek(Token![self]) {
            input.parse::<Token![self]>()?;
            Ok(Self("self".to_owned()))
        } else if input.peek(Token![Self]) {
            input.parse::<Token![Self]>()?;
            Ok(Self("Self".to_owned()))
        } else if input.peek(Token![static]) {
            input.parse::<Token![static]>()?;
            Ok(Self("static".to_owned()))
        } else if input.peek(Token![struct]) {
            input.parse::<Token![struct]>()?;
            Ok(Self("struct".to_owned()))
        } else if input.peek(Token![super]) {
            input.parse::<Token![super]>()?;
            Ok(Self("super".to_owned()))
        } else if input.peek(Token![trait]) {
            input.parse::<Token![trait]>()?;
            Ok(Self("trait".to_owned()))
        } else if input.peek(Token![type]) {
            input.parse::<Token![type]>()?;
            Ok(Self("type".to_owned()))
        } else if input.peek(Token![unsafe]) {
            input.parse::<Token![unsafe]>()?;
            Ok(Self("const".to_owned()))
        } else if input.peek(Token![use]) {
            input.parse::<Token![use]>()?;
            Ok(Self("use".to_owned()))
        } else if input.peek(Token![where]) {
            input.parse::<Token![where]>()?;
            Ok(Self("where".to_owned()))
        } else if input.peek(Token![while]) {
            input.parse::<Token![while]>()?;
            Ok(Self("while".to_owned()))
        } else if input.peek(Token![-]) {
            input.parse::<Token![-]>()?;
            Ok(Self("-".to_owned()))
        } else {
            let id = input.parse::<syn::Ident>()?;
            Ok(Self(id.to_string()))
        }
    }
}
