use crate::err::Diagnostic;
use crate::lex::{lex, preprocess, preprocess_file};
use crate::parser::{Parser, TranslationUnit};
use std::path::Path;

/// Coordinates the C11 preprocessing, lexing, parsing and semantic phases.
pub struct CCompiler {
    source: String,
}

impl CCompiler {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
        }
    }

    pub fn compile(&self) -> Result<TranslationUnit, Vec<Diagnostic>> {
        compile(&self.source)
    }

    pub fn compile_file(path: impl AsRef<Path>) -> Result<TranslationUnit, Vec<Diagnostic>> {
        compile_file(path)
    }
}

/// Compile an in-memory C11 translation unit.
pub fn compile(source: &str) -> Result<TranslationUnit, Vec<Diagnostic>> {
    let source = preprocess(source)?;
    let tokens = lex(&source)?;
    Parser::new(tokens).parse()
}

/// Compile a C11 file and resolve its include directives.
pub fn compile_file(path: impl AsRef<Path>) -> Result<TranslationUnit, Vec<Diagnostic>> {
    let source = preprocess_file(path.as_ref())?;
    let tokens = lex(&source)?;
    Parser::new(tokens).parse()
}
