use proc_macro::{TokenStream, TokenTree};
use proc_macro2::Group;
use quote::quote;
use syn::{
    parse::{self, Parse, ParseBuffer, ParseStream},
    parse2,
    token::{Bracket, Token},
    Ident, Token,
};

#[derive(Debug, Clone)]
pub struct GrammarInput {
    terminals: Vec<SymbolIdent>,
    non_terminals: Vec<SymbolIdent>,
    rules: Vec<RuleInput>,
}

impl GrammarInput {
    pub fn project(&self) -> TokenStream {
        quote! {
            yalp::Grammar::new([
                yalp::Symbol::start(),
                yalp::Symbol::eos(),
                yalp::Symbol::epsilon(),

            ])
        }
        .into()
    }
}

impl Parse for GrammarInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let terminals = Vec::<SymbolIdent>::default();
        let non_terminals = Vec::<SymbolIdent>::default();
        let rules = Vec::<RuleInput>::default();

        loop {
            if input.peek(Ident) && input.peek2(Token![:]) {
                let field = input.parse::<Ident>()?;
                input.parse::<Token![:]>()?;

                if &field.to_string() == "terminals" {
                } else if &field.to_string() == "non_terminals" {
                } else if &field.to_string() == "rules" {
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(Self {
            terminals,
            non_terminals,
            rules,
        })
    }
}

#[derive(Debug, Clone)]
struct RuleInput {
    lhs: SymbolIdent,
    rhs: Vec<SymbolIdent>,
}

#[derive(Debug, Clone)]
struct SymbolIdent(String);

fn parse_symbol_ident_array(input: syn::parse::ParseStream) -> syn::Result<Vec<SymbolIdent>> {
    let group = input.parse::<Group>()?;

    ParseBuffer:quote! {:q}
    parse2(group.stream())
}

fn parse_ident_seq(input: syn::parse::ParseStream) -> syn::Result<String> {
    let mut str = String::default();
    loop {
        if input.peek(Token![-]) {
            input.parse::<Token![-]>()?;
            str.push('-')
        } else if input.peek(Ident) {
            let id = input.parse::<Ident>()?;
            str.push_str(&id.to_string());
        } else {
            break;
        }
    }
    Ok(str)
}

impl Parse for SymbolIdent {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![<]) {
            input.parse::<Token![<]>()?;
            let seq = parse_ident_seq(input)?;
            input.parse::<Token![>]>()?;

            Ok(Self(format!("<{}>", seq)))
        } else {
            Ok(Self(parse_ident_seq(input)?))
        }
    }
}
