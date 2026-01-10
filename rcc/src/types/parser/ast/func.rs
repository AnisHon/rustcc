use crate::types::parser::ast::{DeclKey, StmtKey};
use crate::types::parser::ast::decls::decl::DeclGroup;
use crate::types::parser::declarator::Declarator;
use crate::types::span::Span;

pub type TranslationUnit = Vec<ExternalDecl>;

#[derive(Clone, Debug)]
pub enum ExternalDecl {
    FunctionDefinition(FuncDef),
    Declaration(DeclGroup)
}


pub struct FuncDecl {
    pub declarator: Declarator,
    pub decl_list: Option<Vec<DeclGroup>>, // KR函数的参数
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct FuncDef {
    pub decl: DeclKey,
    pub body: StmtKey,
    pub span: Span
}
