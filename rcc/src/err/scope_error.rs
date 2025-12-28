use thiserror::Error;

use crate::parser::ast::DeclKey;

#[derive(Error, Debug)]
#[error("{msg} {field}")]
pub enum ScopeError {
    #[error("undefined {field}")]
    Undefined { field: &'static str },
    #[error("redefined {field}")]
    Redefined { field: &'static str, prev: DeclKey },
}