use thiserror::Error;

use crate::{
    parser::ast::{DeclKey, StmtKey},
    types::span::Span,
};

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
