//! Declaration nodes and declaration-identity metadata.
//!
//! `DeclId` identifies a particular declaration occurrence. Compatible redeclarations keep their
//! own IDs and link backwards through `previous_declaration`; this is different from TypeId,
//! which identifies a canonical type.

use super::{CType, Expression, FunctionSpecifiers, Parameter, Statement, StorageClass};
use crate::source::SourceRange;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct DeclId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct DeclContextId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Linkage {
    #[default]
    None,
    Internal,
    External,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StorageDuration {
    #[default]
    None,
    Automatic,
    Static,
    Thread,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TranslationUnit {
    pub declarations: Vec<ExternalDeclaration>,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum ExternalDeclaration {
    Declaration(Declaration),
    Function(FunctionDefinition),
    StaticAssert(StaticAssertion),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticAssertion {
    pub condition: Expression,
    pub message: String,
    pub range: SourceRange,
}

#[derive(Debug, Clone, PartialEq)]
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
pub enum Initializer {
    Expression(Expression),
    List(Vec<InitializerItem>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct InitializerItem {
    pub designators: Vec<Designator>,
    pub value: Initializer,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Designator {
    Field(String),
    Index(Expression),
}
