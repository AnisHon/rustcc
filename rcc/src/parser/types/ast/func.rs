use crate::parser::types::ast::decl::{Decl, DeclGroup};
use crate::parser::types::ast::stmt::Stmt;
use crate::types::span::Span;

pub type TranslationUnit = Vec<ExternalDecl>;

#[derive(Clone, Debug)]
pub enum ExternalDecl {
    FunctionDefinition(FuncDef),
    Declaration(DeclGroup)
}

#[derive(Clone, Debug)]
pub struct FuncDef {
    pub func_decl: Decl,
    pub decl_list: Vec<DeclGroup>, // K&R 函数定义
    pub body: Box<Stmt>,
    pub span: Span
}
