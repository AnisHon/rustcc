//! Declaration nodes and declaration-identity metadata.
//!
//! `DeclId` identifies a particular declaration occurrence. Compatible redeclarations keep their
//! own IDs and link backwards through `previous_declaration`; this is different from TypeId,
//! which identifies a canonical type.

use super::{CType, Expression, FunctionSpecifiers, Parameter, Statement, StorageClass};
use crate::source::SourceRange;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
/// Stable identity of one declaration occurrence in the AST.
pub struct DeclId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
/// Identity of a semantic declaration owner, such as a translation unit or function.
pub struct DeclContextId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Whether the declared name denotes the same entity across scopes or files.
pub enum Linkage {
    #[default]
    None,
    Internal,
    External,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Lifetime category of the object or function denoted by a declaration.
pub enum StorageDuration {
    #[default]
    None,
    Automatic,
    Static,
    Thread,
}

#[derive(Debug, Clone, PartialEq)]
/// Root AST node for one preprocessed C translation unit.
pub struct TranslationUnit {
    pub declarations: Vec<ExternalDeclaration>,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
/// Forms permitted at translation-unit scope by the C grammar.
pub enum ExternalDeclaration {
    Declaration(Declaration),
    Function(FunctionDefinition),
    StaticAssert(StaticAssertion),
}

#[derive(Debug, Clone, PartialEq)]
/// A C11 `_Static_assert` after its condition has been parsed and typed.
pub struct StaticAssertion {
    pub condition: Expression,
    pub message: String,
    pub range: SourceRange,
}

#[derive(Debug, Clone, PartialEq)]
/// A bound object, function prototype, typedef, or tag-associated declaration.
///
/// Sema fills identity, redeclaration, context, linkage, duration, and alignment
/// metadata; Parser alone cannot determine those properties.
pub struct Declaration {
    pub id: DeclId,
    pub previous_declaration: Option<DeclId>,
    pub context: DeclContextId,
    pub linkage: Linkage,
    pub storage_duration: StorageDuration,
    pub name: Option<String>,
    pub ty: CType,
    pub storage: StorageClass,
    pub function_specifiers: FunctionSpecifiers,
    pub initializer: Option<Initializer>,
    pub alignment: Option<usize>,
    pub range: SourceRange,
}

#[derive(Debug, Clone, PartialEq)]
/// A function declaration paired with its parameter bindings and body.
pub struct FunctionDefinition {
    pub id: DeclId,
    pub previous_declaration: Option<DeclId>,
    pub context: DeclContextId,
    pub body_context: DeclContextId,
    pub linkage: Linkage,
    pub name: String,
    pub ty: CType,
    pub storage: StorageClass,
    pub function_specifiers: FunctionSpecifiers,
    pub parameters: Vec<Parameter>,
    pub body: Statement,
    pub range: SourceRange,
}

#[derive(Debug, Clone, PartialEq)]
/// Either a scalar expression initializer or a brace-enclosed initializer list.
pub enum Initializer {
    Expression(Expression),
    List(Vec<InitializerItem>),
}

#[derive(Debug, Clone, PartialEq)]
/// One initializer-list element and the optional path it designates.
pub struct InitializerItem {
    pub designators: Vec<Designator>,
    pub value: Initializer,
}

#[derive(Debug, Clone, PartialEq)]
/// A step in a C11 designated-initializer path such as `.member[3]`.
pub enum Designator {
    Field(String),
    Index(Expression),
}
