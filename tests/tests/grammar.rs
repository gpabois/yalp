use yalp::{ConstGrammar, grammar};

const GRAMMAR: ConstGrammar<'static, 9, 6> = grammar! {
    terminals: [0, 1, "+", "*"],
    non_terminals: [E, B],
    rules: {
        <start> => E <eos>;
        E => E "+" B;
        E => E "*" B;
        E => B;
        B => 0;
        B => 1;
    }
};