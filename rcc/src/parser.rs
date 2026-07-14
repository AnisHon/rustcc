//! Recursive-descent C parser with Clang-style parser/Sema separation.
//!
//! Grammar-oriented modules recognize declarations, statements and
//! expressions. `sema` owns scopes, type compatibility, conversions, constant
//! evaluation and layout. The resulting semantic AST is grouped by node kind
//! under `parser::ast`.

pub mod ast;
pub mod parser_core;
mod parser_decl;
mod parser_expr;
mod parser_extern;
mod parser_stmt;
pub(crate) mod sema;

pub use ast::*;
pub use parser_core::Parser;
