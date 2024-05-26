extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;

pub(crate) mod grammar;
pub(crate) mod rule;
pub(crate) mod symbol;

pub(crate) mod lexer;

pub(crate) use grammar::parse_grammar;
pub(crate) use lexer::{Lexer, Token};
pub(crate) use symbol::{parse_symbol_ident_set, SymbolIdentSet};

pub(crate) type Error = ();

/// Declares a new grammar
///
/// # Example
/// ```
/// grammar! {
///     terminals: [<term>, "+", 0, 1],
///     non_terminals: [],
///     rules: {
///         <start> => E <eos>;
///         E => E "+" B;
///         E => B;
///         B => 0;
///         B => 1;
///     }
/// }
/// ```
#[proc_macro]
pub fn grammar(stream: TokenStream) -> TokenStream {
    process_grammar_macro(stream.into()).into()
}

pub(crate) fn process_grammar_macro(stream: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let grammar_input = parse_grammar(stream).unwrap();
    quote! {}.into()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use proc_macro2::TokenStream;

    use super::parse_grammar;

    #[test]
    pub fn test_grammar_macro() {
        let ast = parse_grammar(
            TokenStream::from_str(
                "
            terminals: [E, B, 0, <long-terminal>],
            non_terminals: [],
            rules: {}
        ",
            )
            .expect("cannot parse macro"),
        );

        println!("{:#?}", ast);
    }
}

