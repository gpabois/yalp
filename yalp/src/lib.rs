pub mod ast;
pub mod grammar;
pub mod item;
pub mod lexer;
pub mod lr;
pub mod parser;
pub mod rule;
pub mod symbol;
pub mod token;

pub use grammar::{Grammar, GrammarError, GrammarResult};
pub use item::*;
pub use rule::*;
pub use symbol::*;

mod array;

#[cfg(test)]
pub mod fixtures {
    use crate::{grammar, nterm, rule, sym, term, Grammar, EOS, START};

    pub const FIXTURE_LR1_GRAMMAR: Grammar<'static, 9, 6> = grammar! {
        symbols: [
            term!(s"("),
            term!(s")"),
            term!(n),
            term!(+),
            nterm!(E),
            nterm!(T)
        ],
        rules: [
            rule!(START => sym!{E} EOS),
            rule!(sym!{E} => sym!(T)),
            rule!(sym!{E} => "(" sym!{E} ")"),
            rule!(sym!{E} => sym!{n}),
            rule!(sym!{T} => sym!{+} sym!{T}),
            rule!(sym!{T} => sym!{T} sym!{+} sym!{n})
        ]
    };

    pub const FIXTURE_LR0_GRAMMAR: Grammar<'static, 9, 6> = grammar! {
        symbols: [
            term!(0),
            term!(1),
            term!(*),
            term!(+),
            nterm!(E),
            nterm!(B)
        ],
        rules: [
            rule!(START => sym!(E) EOS),
            rule!(sym!(E) => sym!(E) sym!(*) sym!(B)),
            rule!(sym!(E) => sym!(E) sym!(+) sym!(B)),
            rule!(sym!(E) => sym!(B)),
            rule!(sym!(B) => sym!{0}),
            rule!(sym!(B) => sym!(1))
        ]
    };

    #[test]
    fn test_lr0_grammar() {
        println!("{:#?}", FIXTURE_LR1_GRAMMAR);
        println!("{:#?}", FIXTURE_LR0_GRAMMAR);
    }
}
