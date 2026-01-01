use crate::parser::ast::ExprKey;
use crate::parser::ast::decls::initializer::Initializer;
use crate::parser::semantic::common::Ident;
use crate::parser::semantic::decl_spec::{DeclSpec, ParamDecl, TypeQuals};
use crate::types::span::Span;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Declarator {
    pub name: Option<Ident>,
    pub decl_spec: Rc<DeclSpec>,
    pub chunks: Vec<DeclaratorChunk>,
    pub span: Span,
}

impl Declarator {
    pub fn new(decl_spec: Rc<DeclSpec>) -> Self {
        Self {
            name: None,
            decl_spec,
            chunks: Vec::new(),
            span: Span::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum DeclaratorChunkKind {
    Array { expr: Option<ExprKey> },
    Pointer { type_quals: TypeQuals },
    Function { param: ParamDecl },
}

#[derive(Clone, Debug)]
pub struct DeclaratorChunk {
    pub kind: DeclaratorChunkKind,
    pub span: Span,
}

impl DeclaratorChunk {
    pub fn new(kind: DeclaratorChunkKind, span: Span) -> DeclaratorChunk {
        Self { kind, span }
    }
}

#[derive(Clone, Debug)]
pub struct InitDeclarator {
    pub declarator: Declarator,
    pub init: Option<Initializer>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct InitDeclaratorList {
    pub inits: Vec<InitDeclarator>,
    pub span: Span,
}

impl InitDeclaratorList {
    pub fn new() -> Self {
        Self {
            inits: Vec::new(),
            span: Span::default(),
        }
    }
}

/// decl 解析前缀
pub struct DeclPrefix {
    pub decl_spec: Rc<DeclSpec>,
    pub declarator: Option<Declarator>,
    pub lo: Span,
}
