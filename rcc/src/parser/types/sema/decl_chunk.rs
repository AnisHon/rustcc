use crate::parser::types::ast::decl::{Initializer};
use crate::parser::types::ast::expr::Expr;
use crate::parser::types::common::Ident;
use crate::parser::types::sema::decl_spec::TypeQual;
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
    Array{ l: Span, type_quals: Option<Vec<TypeQual>>, expr: Option<Box<Expr>>, r: Span },
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



#[derive(Clone, Debug)]
pub enum ParamDecl {
    Idents(IdentList),
    Params {
        params: Vec<Declarator>,
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

