use crate::lex::types::token_kind::TokenKind;
use crate::types::span::Span;
use std::fmt::Debug;

/// 词法分析输出Token
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct Token {
    pub span: Span,
    pub kind: TokenKind,
}

impl Token {

    pub fn new(beg: usize, end: usize, kind: TokenKind) -> Self {
        debug_assert!(beg <= end);
        
        Self {
            span: Span::new(beg, end),
            kind,
        }
    }

}











