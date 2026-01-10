use thiserror::Error;

use crate::types::span::Span;
use crate::types::parser::ast::{DeclKey, StmtKey};

pub type ScopeResult<T> = Result<T, ScopeError>;

#[derive(Debug, Clone, Copy)]
pub enum ScopeSource {
    Tag,
    Label,
    Ident,
    Member,
}

#[derive(Debug)]
pub enum ScopeErrorKind {
    Undefined,
    Redefined { prev: DeclKey },
    Conflict { prev: DeclKey },
    RedefinedLabel { prev: StmtKey },
    UndefinedLabel,
}

///
/// # Members
/// - `kind`:
/// - `name`:
/// - `curr`:
/// - `scope`:
/// - `span`:
#[derive(Debug, Error)]
#[error("ScopeError")]
pub struct ScopeError {
    pub kind: ScopeErrorKind,
    pub name: &'static str,
    pub scope: ScopeSource,
    pub span: Span,
}
