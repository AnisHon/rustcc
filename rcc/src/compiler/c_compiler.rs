use super::type_import::TypeImporter;
use crate::err::Diagnostic;
use crate::err::ErrorKind;
use crate::lex::{PPTokenKind, Preprocessor, classify_preprocessed};
use crate::parser::{Parser, TranslationUnit};
use crate::source::{SourceLocation, SourceManager};
use crate::{TargetInfo, TypeContext};
use std::ops::Deref;
use std::path::Path;

#[derive(Debug)]
pub struct Compilation {
    pub source_manager: SourceManager,
    pub target: TargetInfo,
    pub types: TypeContext,
    pub ast: TranslationUnit,
}

impl Deref for Compilation {
    type Target = TranslationUnit;

    fn deref(&self) -> &Self::Target {
        &self.ast
    }
}

#[derive(Debug)]
pub struct CompileError {
    pub source_manager: SourceManager,
    pub diagnostics: Vec<Diagnostic>,
}

impl Deref for CompileError {
    type Target = [Diagnostic];

    fn deref(&self) -> &Self::Target {
        &self.diagnostics
    }
}

impl IntoIterator for CompileError {
    type Item = Diagnostic;
    type IntoIter = std::vec::IntoIter<Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.into_iter()
    }
}

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

    pub fn compile(&self) -> Result<Compilation, CompileError> {
        compile(&self.source)
    }

    pub fn compile_file(path: impl AsRef<Path>) -> Result<Compilation, CompileError> {
        compile_file(path)
    }
}

/// Compile an in-memory C11 translation unit.
pub fn compile(source: &str) -> Result<Compilation, CompileError> {
    let mut sources = SourceManager::new();
    let file = match sources.add_memory_buffer("<memory>", source) {
        Ok(file) => file,
        Err(error) => return Err(compile_error(sources, source_error(error))),
    };
    finish_compilation(sources, file)
}

/// Compile a C11 file and resolve its include directives.
pub fn compile_file(path: impl AsRef<Path>) -> Result<Compilation, CompileError> {
    let mut sources = SourceManager::new();
    let file = match sources.add_file(path.as_ref(), SourceLocation::INVALID) {
        Ok(file) => file,
        Err(error) => return Err(compile_error(sources, source_error(error))),
    };
    finish_compilation(sources, file)
}

fn finish_compilation(
    mut source_manager: SourceManager,
    main_file: crate::source::FileId,
) -> Result<Compilation, CompileError> {
    match compile_source_manager(&mut source_manager, main_file) {
        Ok(mut ast) => {
            let mut types = TypeContext::new();
            TypeImporter::new(&mut types).import_translation_unit(&mut ast);
            Ok(Compilation {
                source_manager,
                target: TargetInfo::default(),
                types,
                ast,
            })
        }
        Err(diagnostics) => Err(compile_error(source_manager, diagnostics)),
    }
}

fn compile_error(source_manager: SourceManager, diagnostics: Vec<Diagnostic>) -> CompileError {
    CompileError {
        source_manager,
        diagnostics,
    }
}

fn compile_source_manager(
    sources: &mut SourceManager,
    main_file: crate::source::FileId,
) -> Result<TranslationUnit, Vec<Diagnostic>> {
    let mut preprocessor = match Preprocessor::new(sources, main_file) {
        Ok(preprocessor) => preprocessor,
        Err(error) => return Err(vec![preprocessor_diagnostic(error)]),
    };
    let mut preprocessing_tokens = Vec::new();
    let preprocessing_result = loop {
        let token = match preprocessor.next_token() {
            Ok(token) => token,
            Err(error) => break Err(error),
        };
        let eof = token.kind == PPTokenKind::EndOfFile;
        preprocessing_tokens.push(token);
        if eof {
            break Ok(preprocessing_tokens);
        }
    };
    drop(preprocessor);
    let preprocessing_tokens =
        preprocessing_result.map_err(|error| vec![preprocessor_diagnostic(error)])?;
    let tokens = classify_preprocessed(preprocessing_tokens)?;
    Parser::new(tokens).parse()
}

fn preprocessor_diagnostic(error: crate::lex::PreprocessorError) -> Diagnostic {
    Diagnostic::new(ErrorKind::Lexical, error.message, error.range)
}

fn source_error(error: crate::source::SourceError) -> Vec<Diagnostic> {
    vec![Diagnostic::new(
        ErrorKind::Lexical,
        error.to_string(),
        crate::source::SourceRange::default(),
    )]
}
