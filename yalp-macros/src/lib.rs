extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{discouraged::AnyDelimiter, Parse},
    parse_macro_input, token, Ident, LitStr, Token,
};

mod ast;

/// The main grammar macro.
///
/// # Example
/// grammar {
///     terminals: [<term>, +, _, 0, 1],
///     non-terminals: [],
///     rules: {
///         <start> => E <eos>;
///         E => ...;
///     }
/// }
#[proc_macro]
pub fn grammar(input: TokenStream) -> TokenStream {
    let grammar: ast::GrammarInput = parse_macro_input!(input);
    grammar.project()
}
