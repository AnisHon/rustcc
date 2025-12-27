use crate::err::scope_error::ScopeError;
use crate::err::type_error::TypeError;
use crate::lex::types::token_kind::Symbol;
use crate::parser::ast::{DeclKey, TypeKey};
use crate::parser::common::Ident;
use crate::types::span::Span;
use core::error;
use std::backtrace::Backtrace;
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
    #[error(
        "Type specifier missing, defaults to 'int'; ISO C99 and later do not support implicit int"
    )]
    TypeSpecifierMissing,
    #[error("Cannot combine with previous '{prev}' {context}")]
    NonCombinable { prev: String, context: String },
    #[error("Duplicate '{item}' {context}")]
    Duplicate { item: String, context: String },
    #[error("Redefinition of '{symbol}'")]
    Redefinition { symbol: &'static str, prev: DeclKey },
    #[error("Undefined '{symbol}'")]
    Undefined { symbol: &'static str },
    #[error("Subscripted value is not an array, pointer or vector")]
    NonSubscripted,
    #[error("No Member named '{field}' in '{ty}'")]
    NoMember { field: String, ty: String },
    #[error("Member reference base type is not a structure or union")]
    NotStructOrUnion { ty: TypeKey },
    #[error("Object Not Callable")]
    UnCallable,
    #[error("{msg}")]
    ErrorMessage { msg: String },
    #[error("is not an integer expression")]
    NotIntConstant,
    #[error("")]
    IntegerTooLarge,
    #[error("")]
    BitFieldExceed {
        max_bit: u64,
        actual_bit: u64,
        field: Option<Symbol>,
    },
    #[error("{err}")]
    TypeError { err: TypeError },
    #[error("Statement requires expression of scalar type")]
    NotScalar { ty: TypeKey },
    #[error("Incompatible operand types")]
    Incompatible { ty1: TypeKey, ty2: TypeKey },
}

impl ErrorKind {
    pub fn redefinition(symbol: Symbol, prev: DeclKey) -> ErrorKind {
        ErrorKind::Redefinition {
            symbol: symbol.get(),
            prev,
        }
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
            | NotStructOrUnion { .. }
            | NotIntConstant
            | IntegerTooLarge
            | BitFieldExceed { .. }
            | TypeError { .. }
            | NotScalar { .. }
            | Incompatible { .. } => Error,
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
    pub error_kind: ErrorKind, // 错误信息
    pub level: ErrorLevel,
    pub backtrace: Backtrace,
    pub span: Span,
}

impl ParserError {
    pub fn new(error_kind: ErrorKind, span: Span) -> Self {
        // should be close
        let backtrace = Backtrace::capture();
        let level = ErrorLevel::from_kind(&error_kind);
        Self {
            span,
            error_kind,
            backtrace,
            level,
        }
    }

    pub fn undefined_symbol(ident: &Ident) -> Self {
        let backtrace = Backtrace::capture();
        let level = ErrorLevel::Error;
        let kind = ErrorKind::Undefined {
            symbol: ident.symbol.get(),
        };
        Self {
            span: ident.span,
            error_kind: kind,
            backtrace,
            level,
        }
    }

    pub fn error(msg: String, span: Span) -> Self {
        let kind = ErrorKind::ErrorMessage { msg };
        Self::new(kind, span)
    }

    pub fn not_scalar_type(ty: TypeKey, span: Span) -> Self {
        let kind = ErrorKind::NotScalar { ty };
        Self::new(kind, span)
    }

    pub fn non_subscripted(span: Span) -> Self {
        let kind = ErrorKind::NonSubscripted;
        Self::new(kind, span)
    }

    pub fn integer_too_large(span: Span) -> Self {
        let kind = ErrorKind::IntegerTooLarge;
        Self::new(kind, span)
    }

    pub fn not_int_constant(span: Span) -> Self {
        let kind = ErrorKind::NotIntConstant;
        Self::new(kind, span)
    }

    pub fn bit_field_exceed(
        max_bit: u64,
        actual_bit: u64,
        field: Option<Symbol>,
        span: Span,
    ) -> Self {
        let kind = ErrorKind::BitFieldExceed {
            max_bit,
            actual_bit,
            field,
        };
        Self::new(kind, span)
    }

    pub fn duplicate(item: String, ctx: String, span: Span) -> Self {
        let kind = ErrorKind::Duplicate { item, context: ctx };
        Self::new(kind, span)
    }

    pub fn non_combinable(prev: String, ctx: String, span: Span) -> Self {
        let kind = ErrorKind::NonCombinable { prev, context: ctx };
        Self::new(kind, span)
    }

    pub fn incompatable(ty1: TypeKey, ty2: TypeKey, span: Span) -> Self {
        let kind = ErrorKind::Incompatible { ty1, ty2 };
        Self::new(kind, span)
    }

    pub fn from_scope_error(error: ScopeError, span: Span) -> Self {
        let kind = match error {
            ScopeError::Redefined { field, prev } => ErrorKind::Redefinition {
                symbol: field,
                prev,
            },
            ScopeError::Undefined { field } => ErrorKind::Undefined { symbol: field },
        };
        Self::new(kind, span)
    }

    pub fn from_type_error(error: TypeError, span: Span) -> Self {
        let kind = ErrorKind::TypeError { err: error };
        Self::new(kind, span)
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.level, self.error_kind)
    }
}
