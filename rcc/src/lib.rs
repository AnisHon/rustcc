//! C11 compiler front end.

pub mod compiler;
pub mod constant_eval;
pub mod err;
pub mod lex;
pub mod parser;
pub mod source;
pub mod target;
pub mod type_system;
pub mod writer;

pub use compiler::{
    CCompiler, Compilation, CompileError, compile, compile_file, compile_with_target,
};
pub use constant_eval::{ConstantEvaluator, ConstantValue, EvaluationContext, EvaluationFailure};
pub use err::{Diagnostic, ErrorKind};
pub use lex::{Keyword, Literal, StringEncoding, Token, TokenKind, classify_preprocessed};
pub use parser::ast::*;
pub use source::{
    FileId, FileLocation, PresumedLocation, SourceLocation, SourceManager, SourceRange,
};
pub use target::TargetInfo;
pub use type_system::{
    ArrayBound, BuiltinType, CallingConvention, EnumId, FunctionType, QualType, Qualifiers,
    RecordId, RecordKind, TypeContext, TypeError, TypeId, TypeKind as CanonicalTypeKind,
};
pub use writer::AstWriter;
