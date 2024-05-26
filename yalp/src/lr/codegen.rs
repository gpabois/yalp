use ruast::{Array, Call, Expr, Lit, Path, PathSegment, Tuple};

use crate::{grammar::traits::Grammar, traits::SymbolSlice};

use super::{traits::LrTable, Action};

/// Generate the table value.
pub fn codegen_table_value<'sid, T: LrTable, G: Grammar<'sid>>(grammar: &G, table: &T) -> Expr {
    let rows = (0..table.len())
        .into_iter()
        .map(|state| gen_row_value(state, grammar, table));

    Call::new(
        Path::new(vec![
            PathSegment::simple("yalp"),
            PathSegment::simple("lr"),
            PathSegment::simple("table"),
            PathSegment::simple("codegen"),
            PathSegment::simple("LrTable"),
            PathSegment::simple("new"),
        ]),
        vec![Array::new(rows.collect()).into()],
    )
    .into()
}

pub fn gen_row_value<'sid, T: LrTable, G: Grammar<'sid>>(
    state: usize,
    grammar: &G,
    table: &T,
) -> Expr {
    let actions = grammar.iter_terminals().map(|sym| {
        Tuple::new(vec![
            Lit::str(sym.id).into(),
            table
                .action(state, &sym)
                .map(|action| {
                    Call::new(Path::single("Some"), vec![gen_action_value(action)]).into()
                })
                .unwrap_or(Path::single("None").into()),
        ])
        .into()
    });

    let goto = grammar.iter_non_terminals().map(|sym| {
        Tuple::new(vec![
            Lit::str(sym.id).into(),
            table
                .goto(state, &sym)
                .map(|goto| {
                    Call::new(
                        Path::single("Some"),
                        vec![Lit::uint(goto.to_string()).into()],
                    )
                    .into()
                })
                .unwrap_or(Path::single("None").into()),
        ])
        .into()
    });

    Call::new(
        Path::new(vec![
            PathSegment::simple("yalp"),
            PathSegment::simple("lr"),
            PathSegment::simple("table"),
            PathSegment::simple("codegen"),
            PathSegment::simple("LrTableRow"),
            PathSegment::simple("new"),
        ]),
        vec![
            Array::new(actions.collect()).into(),
            Array::new(goto.collect()).into(),
        ],
    )
    .into()
}

pub fn gen_action_value(action: &Action) -> Expr {
    match action {
        Action::Shift(state) => Call::new(
            Path::new(vec![
                PathSegment::simple("yalp"),
                PathSegment::simple("lr"),
                PathSegment::simple("Action"),
                PathSegment::simple("Shift"),
            ]),
            vec![Lit::uint(state.to_string()).into()],
        )
        .into(),
        Action::Reduce(rule) => Call::new(
            Path::new(vec![
                PathSegment::simple("yalp"),
                PathSegment::simple("lr"),
                PathSegment::simple("Action"),
                PathSegment::simple("Reduce"),
            ]),
            vec![Lit::uint(rule.to_string()).into()],
        )
        .into(),
        Action::Accept => Path::new(vec![
            PathSegment::simple("yalp"),
            PathSegment::simple("lr"),
            PathSegment::simple("Action"),
            PathSegment::simple("Accept"),
        ])
        .into(),
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{fixtures::FIXTURE_LR1_GRAMMAR, LrTable};

    use super::codegen_table_value;

    #[test]
    fn test_codegen_table_value() {
        let grammar = &FIXTURE_LR1_GRAMMAR;
        let table = LrTable::build::<0, _>(grammar).expect("cannot build table");

        let ast = codegen_table_value(grammar, &table);
        println!("{ast}");
    }
}
