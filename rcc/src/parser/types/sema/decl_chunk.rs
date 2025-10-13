use crate::parser::types::ast::decl::{FuncSpec, Initializer, StorageSpec, TypeQual, TypeSpec};
use crate::parser::types::ast::expr::Expr;
use crate::parser::types::common::Ident;
use crate::types::span::Span;

#[derive(Clone, Debug)]
pub struct Declarator {

}

#[derive(Clone, Debug)]
pub struct PointerChunk {
    pub type_quals: Vec<TypeQual>,
    pub span: Span
}

impl PointerChunk {
    pub fn new(type_quals: Vec<TypeQual>, span: Span) -> Self {
        Self { type_quals, span }
    }
}

#[derive(Clone, Debug)]
pub enum DeclChunkKind {
    Ident(Ident),
    Paren{ l: Span, declarator: Declarator, r: Span },
    Array{ l: Span, type_quals: Vec<TypeQual>, expr: Box<Expr>, r: Span },
    Function{ l: Span, param: ParamDecl, r: Span },
}

#[derive(Clone, Debug)]
pub struct DeclChunk {
    pub kind: DeclChunkKind,
    pub span: Span
}

impl DeclChunk {
    pub fn new(kind: DeclChunkKind, span: Span) -> DeclChunk {
        Self { kind, span }
    }
}

pub struct DeclSpec {
    pub storages: Vec<StorageSpec>,
    pub type_specs: Vec<TypeSpec>,
    pub type_quals: Vec<TypeQual>,
    pub func_specs: Vec<FuncSpec>,
    pub span: Span
}

impl DeclSpec {
    pub fn new() -> Self {
        Self {
            storages: Vec::new(),
            type_specs: Vec::new(),
            type_quals: Vec::new(),
            func_specs: Vec::new(),
            span: Span::default()
        }
    }
}

pub struct SpecQualList {
    pub type_specs: Vec<TypeSpec>,
    pub type_quals: Vec<TypeQual>,
    pub span: Span
}

impl SpecQualList {
    pub fn new() -> Self {
        Self {
            type_specs: Vec::new(),
            type_quals: Vec::new(),
            span: Span::default()
        }
    }
}

#[derive(Clone, Debug)]
pub enum ParamDecl {
    Idents(Vec<Ident>),
    Params {
        params: Vec<ParamDecl>,
        commas: Vec<Span>,
        ellipsis: Option<Span>,
    },
}

#[derive(Clone, Debug)]
pub struct StructOrUnionSpec {

}

#[derive(Clone, Debug)]
pub struct EnumSpec {

}

#[derive(Clone, Debug)]

pub struct IdentList {
    pub idents: Vec<Ident>,
    pub commas: Vec<Span>,
    pub span: Span
}

impl IdentList {
    pub fn new() -> Self {
        Self {
            idents: Vec::new(),
            commas: Vec::new(),
            span: Span::default()
        }
    }
}

#[derive(Clone, Debug)]
pub struct InitDeclarator {
    pub declarator: Declarator,
    pub eq: Option<Span>,
    pub init: Option<Initializer>,
}

#[derive(Clone, Debug)]
pub struct InitDeclaratorList {
    pub inits: Vec<InitDeclarator>,
    pub commas: Vec<Span>,
    pub span: Span
}

impl InitDeclaratorList {
    pub fn new() -> Self {
        Self { inits: Vec::new(), commas: Vec::new(), span: Span::default() }
    }
}

#[derive(Clone, Debug)]
pub struct InitializerList {
    pub inits: Vec<Initializer>,
    pub commas: Vec<Span>,
    pub span: Span
}

impl InitializerList {
    pub fn new() -> Self {
        Self { inits: Vec::new(), commas: Vec::new(), span: Span::default() }
    }
}

