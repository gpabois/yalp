use crate::symbol::SymbolName;

pub trait IntoSymbol<'syntax> {
    fn into_symbol(self) -> SymbolName<'syntax>;
}
