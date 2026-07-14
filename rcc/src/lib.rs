//! C11 compiler front end.

pub mod compiler;
pub mod err;
pub mod lex;
pub mod parser;
pub mod types;
pub mod writer;

pub use compiler::{CCompiler, compile, compile_file};
pub use err::{Diagnostic, ErrorKind};
pub use lex::{
    Keyword, Literal, StringEncoding, Token, TokenKind, lex, preprocess, preprocess_file,
};
pub use parser::ast::*;
pub use types::{Position, Span};
pub use writer::AstWriter;
