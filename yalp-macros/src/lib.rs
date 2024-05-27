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
pub(crate) use rule::{parse_rule_set, RuleSet, Rule};

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
    parse_grammar(stream).unwrap().into_token_stream()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use proc_macro2::TokenStream;

    use super::{parse_grammar, process_grammar_macro};

    #[test]
    pub fn test_grammar_macro() {
        let stream = TokenStream::from_str("
            terminals: [E, B, 0, <long-terminal>],
            non_terminals: [],
            rules: {
                <start> => E <eos>;
            }
        ").expect("cannot parse macro");

        let ast = process_grammar_macro(stream);

        println!("{ast}");
    }
}
