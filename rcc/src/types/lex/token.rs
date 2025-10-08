use crate::types::lex::token_kind::TokenKind;
use crate::types::span::Span;
use enum_as_inner::EnumAsInner;
use std::fmt::{Debug, Display, Formatter};

/// 词法分析输出Token
#[derive(Debug, Clone)]
pub struct Token {
    pub span: Span,
    pub kind: TokenKind,
}

impl Token {

    pub fn new(beg: usize, end: usize, kind: TokenKind) -> Self {
        assert!(beg <= end);
        
        Self {
            span: Span::new(beg, end),
            kind,
        }
    }

}











