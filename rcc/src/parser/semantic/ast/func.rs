use crate::parser::ast::decl::DeclKey;
use crate::parser::semantic::ast::decl::{DeclGroup};
use crate::parser::semantic::ast::stmt::Stmt;
use crate::parser::semantic::declarator::Declarator;
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
    pub body: Box<Stmt>,
    pub span: Span
}
