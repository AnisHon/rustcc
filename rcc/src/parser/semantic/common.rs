use crate::lex::types::token::Token;
use crate::lex::types::token_kind::Symbol;
use crate::types::span::{Pos, Span};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
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


#[derive(Clone, Debug)]

pub struct IdentList {
    pub idents: Vec<Ident>,
    pub commas: Vec<Pos>,
    pub span: Span
}

impl IdentList {
    pub fn new() -> Self {
        Self {
            idents: Vec::new(),
            commas: Vec::new(),
            span: Span::default()
        }
    }
}