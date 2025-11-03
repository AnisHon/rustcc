use crate::parser::semantic::ast::decl::Initializer;
use crate::parser::semantic::ast::expr::Expr;
use crate::parser::semantic::common::Ident;
use crate::parser::semantic::decl_spec::{DeclSpec, ParamDecl, TypeQualType};
use crate::types::span::{Pos, Span};
use std::rc::Rc;
use crate::parser::semantic::sema::decl::decl_context::DeclContextRef;

#[derive(Clone, Debug)]
pub struct Declarator {
    pub name: Option<Ident>,
    pub decl_spec: Rc<DeclSpec>,
    pub chunks: Vec<DeclaratorChunk>,
    pub span: Span
}

impl Declarator {
    pub fn new(decl_spec: Rc<DeclSpec>) -> Self {
        Self {
            name: None,
            decl_spec,
            chunks: Vec::new(),
            span: Span::default()
        }
    }
}


#[derive(Clone, Debug)]
pub enum DeclaratorChunkKind {
    Paren { l: Pos, r: Pos }, // 纯用来保存括号信息了
    Array { l: Pos, type_qual: Option<TypeQualType>, expr: Option<Box<Expr>>, r: Pos },
    Pointer { star: Pos, type_qual: TypeQualType },
    Function { l: Pos, param: ParamDecl, r: Pos },
}

#[derive(Clone, Debug)]
pub struct DeclaratorChunk {
    pub kind: DeclaratorChunkKind,
    pub span: Span
}

impl DeclaratorChunk {
    pub fn new(kind: DeclaratorChunkKind, span: Span) -> DeclaratorChunk {
        Self { kind, span }
    }
}

#[derive(Clone, Debug)]
pub struct InitDeclarator {
    pub declarator: Declarator,
    pub eq: Option<Pos>,
    pub init: Option<Initializer>,
    pub span: Span
}

#[derive(Clone, Debug)]
pub struct InitDeclaratorList {
    pub inits: Vec<InitDeclarator>,
    pub commas: Vec<Pos>,
    pub span: Span
}

impl InitDeclaratorList {
    pub fn new() -> Self {
        Self { inits: Vec::new(), commas: Vec::new(), span: Span::default() }
    }
}

pub struct FunctionDeclarator {

}