use super::{CType, Expression, FunctionSpecifiers, Parameter, Statement, StorageClass};
use crate::source::SourceRange;

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
