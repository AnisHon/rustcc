//! Interned C types shared by AST and semantic analysis.
//!
//! This module deliberately has no dependency on parsing or semantic state. Declarations own
//! record completion and VLA bound expressions; types only retain stable identities and shape.

mod context;
mod kind;

pub use context::{TypeContext, TypeError};
pub use kind::{
    ArrayBound, BuiltinType, CallingConvention, EnumId, FunctionType, QualType, Qualifiers,
    RecordId, RecordKind, TypeId, TypeKind,
};
