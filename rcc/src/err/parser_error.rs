use crate::types::span::{Pos, Span};
use std::fmt::{Display, Formatter};
use thiserror::Error;

pub type ParserResult<T> = Result<T, ParserError>;

#[derive(Error, Debug)]
pub enum ErrorKind {
    #[error("expect {expect} found {found}")]
    ExpectButFound { expect: String, found: String },
    #[error("expect {expect}")]
    Expect { expect: String },
    #[error("{ty} is not assignable")]
    NotAssignable { ty: String },
    #[error("Type specifier missing, defaults to 'int'; ISO C99 and later do not support implicit int")]
    TypeSpecifierMissing,
    #[error("Cannot combine with previous '{prev}' {context}")]
    NonCombinable { prev: String, context: String },
    #[error("Duplicate '{item}' {context}")]
    Duplicate { item: String, context: String },
}
#[derive(Debug)]
pub enum ErrorLevel {
    Note,
    Warning,
    Error,
}

impl ErrorLevel {
    fn from_kind(kind: &ErrorKind) -> Self {
        use ErrorKind::*;
        use ErrorLevel::*;
        match kind {
            ExpectButFound { .. }
            | Expect { .. }
            | NotAssignable { .. } 
            | TypeSpecifierMissing 
            | NonCombinable { .. } => Error,
            Duplicate { .. } => Warning,
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