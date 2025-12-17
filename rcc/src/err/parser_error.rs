use std::backtrace::Backtrace;
use crate::lex::types::token_kind::Symbol;
use crate::parser::common::Ident;
use crate::types::span::Span;
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
    #[error("Redefinition of '{symbol}'")]
    Redefinition { symbol: String },
    #[error("Undefined '{symbol}'")]
    Undefined { symbol: &'static str },
    #[error("Subscripted value is not an array,pointer,or vector")]
    NonSubscripted,
    #[error("No Member named '{field}' in '{ty}'")]
    NoMember { field: String, ty: String },
    #[error("Member reference base type '{ty}' is not a structure or union")]
    NotStructOrUnion {ty: String},
    #[error("Object Not Callable")]
    UnCallable,
    #[error("{msg}")]
    ErrorMessage{ msg: String },
}

impl ErrorKind {
    pub fn redefinition(symbol: Symbol) -> ErrorKind {
        let symbol = symbol.get().to_owned();
        ErrorKind::Redefinition { symbol }
    }
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
            | NonSubscripted { .. }
            | UnCallable { .. }
            | Expect { .. }
            | NotAssignable { .. } 
            | TypeSpecifierMissing 
            | NonCombinable { .. }
            | Undefined { .. }
            | Redefinition { .. }
            | NoMember { .. }
            | ErrorMessage { .. }
            | NotStructOrUnion { .. } => Error,
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
    pub error_kind: ErrorKind,    // 错误信息
    pub level: ErrorLevel,
    pub backtrace: Backtrace,
    pub span: Span,
}

impl ParserError {
    pub fn new(error_kind: ErrorKind, span: Span) -> Self {
        // should be close
        let backtrace = Backtrace::capture();
        let level = ErrorLevel::from_kind(&error_kind);
        Self { span, error_kind, backtrace, level }
    }

    pub fn undefined_symbol(ident: &Ident) -> Self {
        let backtrace = Backtrace::capture();
        let level = ErrorLevel::Error;
        let kind = ErrorKind::Undefined { symbol: ident.symbol.get() };
        Self { span: ident.span, error_kind: kind, backtrace, level  }
    }

    pub fn error(msg: String, span: Span) -> Self {
        let kind = ErrorKind::ErrorMessage { msg };
        Self::new(kind, span)
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.level ,self.error_kind)
    }
}