use crate::symbol::Symbol;

pub trait IntoSymbol<'syntax> {
    fn into_symbol(self) -> Symbol<'syntax>;
}
