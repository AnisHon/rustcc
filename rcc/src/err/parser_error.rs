use std::error::Error;
use std::fmt::{Display, Formatter};
use thiserror::Error;
use crate::lex::types::token::Token;
use crate::types::span::Span;

pub type ParserResult<T> = Result<T, ParserError>;

#[derive(Error, Debug)]
pub enum ErrorKind {
    #[error("expect {expect} found {found}")]
    ExpectButFound { expect: String, found: String },
    #[error("expect {expect}")]
    Expect { expect: String },
}
#[derive(Debug)]
pub enum ErrorLevel {
    Note,
    Warning,
    Error,
}

impl ErrorLevel {
    fn from_kind(kind: &ErrorKind) -> Self {
        match kind {
            ErrorKind::ExpectButFound { .. }
            | ErrorKind::Expect { .. }=> ErrorLevel::Error,
        }
    }
}

impl Display for ErrorLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            ErrorLevel::Note => "note",
            ErrorLevel::Warning => "warning",
            ErrorLevel::Error => "error",
        };
        write!(f, "{}", msg)
    }
}

#[derive(Debug)]
pub struct ParserError {
    pub span: Span,
    pub error_kind: ErrorKind,    // 错误信息
    pub level: ErrorLevel
}

impl ParserError {
    pub fn new(span: Span, error_kind: ErrorKind) -> Self {
        let level = ErrorLevel::from_kind(&error_kind);
        Self { span, error_kind, level }
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.level ,self.error_kind)
    }
}