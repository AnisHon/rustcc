//! Frontend orchestration and ownership of per-compilation state.
//!
//! The phase implementations live in their own modules. This module only wires them together
//! and makes sure location/type handles never outlive the managers that allocated them.

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
/// A successful frontend invocation.
///
/// `ast` contains compact source and canonical-type handles, so the corresponding managers are
/// intentionally returned in the same object. Keeping only the AST would make those handles
/// impossible to resolve.
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
/// Diagnostics together with the source buffers needed to render them.
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
    target: TargetInfo,
}

impl CCompiler {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            target: TargetInfo::default(),
        }
    }

    pub fn with_target(mut self, target: TargetInfo) -> Self {
        self.target = target;
        self
    }

    pub fn compile(&self) -> Result<Compilation, CompileError> {
        compile_with_target(&self.source, self.target.clone())
    }

    pub fn compile_file(path: impl AsRef<Path>) -> Result<Compilation, CompileError> {
        compile_file(path)
    }
}

/// Compile an in-memory C11 translation unit.
pub fn compile(source: &str) -> Result<Compilation, CompileError> {
    compile_with_target(source, TargetInfo::default())
}

pub fn compile_with_target(source: &str, target: TargetInfo) -> Result<Compilation, CompileError> {
    let mut sources = SourceManager::new();
    let file = match sources.add_memory_buffer("<memory>", source) {
        Ok(file) => file,
        Err(error) => return Err(compile_error(sources, source_error(error))),
    };
    finish_compilation(sources, file, target)
}

/// Compile a C11 file and resolve its include directives.
pub fn compile_file(path: impl AsRef<Path>) -> Result<Compilation, CompileError> {
    let mut sources = SourceManager::new();
    let file = match sources.add_file(path.as_ref(), SourceLocation::INVALID) {
        Ok(file) => file,
        Err(error) => return Err(compile_error(sources, source_error(error))),
    };
    finish_compilation(sources, file, TargetInfo::default())
}

fn finish_compilation(
    mut source_manager: SourceManager,
    main_file: crate::source::FileId,
    target: TargetInfo,
) -> Result<Compilation, CompileError> {
    match compile_source_manager(&mut source_manager, main_file, &target) {
        Ok(mut ast) => {
            // Declarators are easiest to build as recursive types. Once Sema has accepted the
            // translation unit, replace those structural identities with interned TypeIds.
            let mut types = TypeContext::new();
            TypeImporter::new(&mut types).import_translation_unit(&mut ast);
            Ok(Compilation {
                source_manager,
                target,
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
    target: &TargetInfo,
) -> Result<TranslationUnit, Vec<Diagnostic>> {
    // Preprocessor borrows SourceManager mutably because includes and macro expansions allocate
    // new buffers/location records. Drain it before language-token classification and parsing.
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
    Parser::with_target(tokens, target.clone()).parse()
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
