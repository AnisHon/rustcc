use std::sync::{mpsc, Arc};
use crate::content_manager::ContentManager;
use crate::lex::lex_core::{run_lexer, Lex};
use crate::lex::types::token_kind::TokenKind;
use crate::parser::parser_core::Parser;

mod parser_decl;
mod parser_expr;
pub mod parser_core;
mod semantic;
mod parser_stmt;
mod parser_function;

pub use crate::parser::semantic::{ast, sema::Sema, common};
