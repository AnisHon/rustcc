mod ast;
mod error;
mod lexer;
mod parser;
mod preprocessor;
mod token;

pub use ast::*;
pub use error::{Diagnostic, ErrorKind};
pub use lexer::lex;
pub use preprocessor::{preprocess, preprocess_file};
pub use token::{Keyword, Literal, Token, TokenKind};

/// Lex, parse and semantically type a C translation unit.
pub fn compile(source: &str) -> Result<TranslationUnit, Vec<Diagnostic>> {
    let source = preprocess(source)?;
    let tokens = lex(&source)?;
    parser::Parser::new(tokens).parse()
}

/// Compile a C file, resolving quoted and system includes before parsing.
pub fn compile_file(path: impl AsRef<std::path::Path>) -> Result<TranslationUnit, Vec<Diagnostic>> {
    let source = preprocess_file(path.as_ref())?;
    let tokens = lex(&source)?;
    parser::Parser::new(tokens).parse()
}
