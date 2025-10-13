use crate::lex::types::token_kind::Symbol;
use crate::types::span::Span;

#[derive(Clone, Debug)]
pub struct Ident {
    pub symbol: Symbol,
    pub span: Span,
}