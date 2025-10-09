use std::error::Error;
use std::fmt::{Display, Formatter};
use crate::lex::types::token::Token;
use crate::types::span::Span;

pub type ParserResult<T> = Result<T, ParserError>;

/// 同时返回数据和错误
pub struct PartialResult<T> {
    pub data: T,
    pub errors: Vec<ParserError>,
}

impl<T> PartialResult<T> {
    pub fn new(data: T, errors: Vec<ParserError>) -> Self {
        Self {
            data,
            errors
        }
    }
    pub fn merge_error(&mut self, errors: &mut Vec<ParserError>) {
        self.errors.append(errors)
    }
    
}

impl<T: Default> Default for PartialResult<T> {
    fn default() -> Self {
        Self {
            data: T::default(),
            errors: vec![],
        }
    }
}

#[derive(Debug)]
pub enum ErrorLevel {
    Note,
    Warning,
    Error,
}

#[derive(Debug)]
pub struct ParserError {
    pub span: Span,
    pub msg: String,    // 错误信息
    pub level: ErrorLevel
}

impl ParserError {
    pub fn new(span: Span, msg: &str, name: &'static str) -> Self {
        Self { span, msg: msg.to_string(), level: ErrorLevel::Error }
    }

    pub fn error(span: Span, msg: String) -> Self {
        Self { span, msg, level: ErrorLevel::Error }
    }
    
    pub fn warning(span: Span, msg: String) -> Self {
        Self { span, msg, level: ErrorLevel::Warning }
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