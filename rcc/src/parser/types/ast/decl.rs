use crate::lex::types::token::Token;
use crate::lex::types::token_kind::{Keyword, TokenKind};
use crate::parser::types::ast::expr::Expr;
use crate::parser::types::common::Ident;
use crate::parser::types::sema::decl_chunk::InitializerList;
use crate::types::span::Span;

#[derive(Debug, Clone)]
pub enum Initializer {
    Expr(Box<Expr>),
    InitList{ l: Span, inits: InitializerList, r: Span },
}

#[derive(Debug, Clone)]
pub struct Decl {
    pub ident: Ident,
    pub eq: Span,
    pub init: Option<Initializer>,
    pub semi: Span,
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

pub enum StructOrUnionDeclKind {
    Ref {
        ident: Ident,
    },
    Decl {
        ident: Option<Ident>,
        l: Span,
        fields: Vec<StructField>,
        r: Span,
    }
}

pub struct StructOrUnionDecl {
    struct_or_union: StructOrUnion,
    kind: StructOrUnionDeclKind,
    span: Span,
}

pub struct StructField {
    pub ident: Option<Ident>,
    pub colon: Option<Token>,
    pub bit_field: Option<Box<Expr>>,
    pub semi: Span,
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
