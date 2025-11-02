use std::rc::Rc;
use crate::parser::types::ast::decl::{Decl, DeclGroup};
use crate::parser::types::ast::stmt::Stmt;
use crate::parser::types::common::Ident;
use crate::parser::types::declarator::Declarator;
use crate::parser::types::sema::sema_type::Type;
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
    pub decl: Rc<Decl>,
    pub body: Box<Stmt>,
    pub span: Span
}
