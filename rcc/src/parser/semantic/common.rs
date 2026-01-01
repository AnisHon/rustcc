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
        Self { symbol, span }
    }
}

#[derive(Clone, Debug)]

pub struct IdentList {
    pub idents: Vec<Ident>,
    pub span: Span,
}

impl IdentList {
    pub fn new() -> Self {
        Self {
            idents: Vec::new(),
            span: Span::default(),
        }
    }
}

/// 状态机状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeSpecState {
    Init,
    Void,
    Char,
    Short,
    Int,
    Long,
    LongLong,
    Float,
    Double,
    LongDouble,
    Record,
    Enum,
    TypeName,
}

impl TypeSpecState {
    pub fn combine(state1: TypeSpecState, state2: TypeSpecState) -> Option<TypeSpecState> {
        use TypeSpecState::*;
        match (state1, state2) {
            (Init, _) => Some(state2),
            (Void, _) => None,
            (Char, Int) => Some(Char),
            (Short, Int) => Some(Short),
            (Int, Char) => Some(Char),
            (Int, Short) => Some(Short),
            (Int, Long) => Some(Int),
            (Int, LongLong) => Some(LongLong),
            (Long, Int) => Some(Long),
            (Long, Long) => Some(LongLong),
            (Long, Double) => Some(LongDouble),
            (LongLong, Int) => Some(LongLong),
            (Float, _) => None,
            (Double, Long) => Some(LongDouble),
            (LongDouble, _) => None,
            (Record, _) => None,
            (Enum, _) => None,
            (TypeName, _) => None,
            (_, _) => None,
        }
    }
}
