use crate::lex::types::token::Token;
use crate::lex::types::token_kind::Symbol;
use crate::types::span::Span;

#[derive(Clone, Debug)]
pub struct Ident {
    pub symbol: Symbol,
    pub span: Span,
}

impl Ident {
    pub fn new(token: Token) -> Self {
        let symbol = token.kind.into_ident().unwrap();
        let span = token.span;
        Self {
            symbol,
            span
        }
    }
}