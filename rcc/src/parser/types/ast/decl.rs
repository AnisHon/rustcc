use crate::lex::types::token::Token;
use crate::parser::types::ast::expr::Expr;
use crate::parser::types::common::Ident;
use crate::types::span::{Pos, Span};

#[derive(Debug, Clone)]
pub enum Initializer {
    Expr(Box<Expr>),
    InitList{ l: Pos, inits: InitializerList, r: Pos },
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

#[derive(Debug, Clone)]
pub struct Decl {
    pub ident: Ident,
    pub eq: Pos,
    pub init: Option<Initializer>,
    pub semi: Pos,
    pub span: Span,
}

impl Decl {
    // pub fn new(ident: Token, eq: Token, ) -> Span {
    //
    // }
}

pub enum StructOrUnionKind {
    Struct,
    Union,
}

pub struct StructOrUnion {
    pub kind: StructOrUnionKind,
    pub span: Span,
}

pub enum StructDeclKind {
    Ref {
        ident: Ident,
    },
    Decl {
        ident: Option<Ident>,
        l: Span,
        fields: Vec<StructVarDecl>,
        r: Span,
    }
}

// struct or union
pub struct StructDecl {
    pub struct_or_union: StructOrUnion,
    pub kind: StructDeclKind,
    pub span: Span,
}

pub struct StructVarDecl {
    pub ident: Option<Ident>,
    pub colon: Option<Token>,
    pub bit_field: Option<Box<Expr>>,
    pub semi: Pos,
}



// pub struct EnumeratorList {
//     pub enums: Vec<Enumerator>,
//     pub commas: Vec<Span>,
// }
//
// pub struct EnumDecl {
//     pub enum_span: Span,
//     pub l: Span,
//     pub enums: EnumeratorList,
//     pub r: Span,
//     pub span: Span
// }
