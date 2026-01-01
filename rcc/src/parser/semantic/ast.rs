pub mod common;
pub mod decl;
mod decls;
pub mod exprs;
pub mod func;
pub mod stmt;
pub mod types;
pub mod visitor;

// 重新导出几个 key
pub use common::{DeclKey, ExprKey, StmtKey, TypeKey};
