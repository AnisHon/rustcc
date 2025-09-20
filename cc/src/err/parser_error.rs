use std::error::Error;
use std::fmt::{Display, Formatter};
use crate::types::lex::token::Token;
use crate::types::span::Span;

pub type ParserResult<T> = Result<T, ParserError>;

#[derive(Debug)]
pub struct ParserError {
    pub span: Span,
    pub name: String,   // 错误文法名称
    pub msg: String,    // 错误信息

}

impl ParserError {
    pub fn new(span: Span, msg: &str, name: &'static str) -> Self {
        Self { span, msg: msg.to_string(), name: name.to_string() }
    }

    /// todo 可能以后不需要这个
    pub fn with_pos(mut self, pos: usize) -> Self {
        self.span.start = pos;
        self
    }

    /// todo 可能以后不需要这个
    pub fn with_line(mut self, end: usize) -> Self {
        self.span.end = end;
        self
    }
    
    pub fn with_name(mut self, name: &'static str) -> Self {
        self.name = name.to_string();
        self
    }

    /// todo 可能以后不需要这个
    pub fn with_token(mut self, token: &Token) -> Self {
        self.span = token.span;
        self
    }
    pub fn with_msg(mut self, msg: &'static str) -> Self {
        self.msg = msg.to_string();
        self
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ParserError {

}