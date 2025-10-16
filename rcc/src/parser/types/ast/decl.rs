use std::rc::Rc;
use crate::lex::types::token::Token;
use crate::lex::types::token_kind::Keyword;
use crate::parser::types::ast::expr::Expr;
use crate::parser::types::common::Ident;
use crate::parser::types::sema::sema_type::Type;
use crate::types::span::{Pos, Span};

#[derive(Debug, Clone)]
pub enum Initializer {
    Expr(Box<Expr>),
    InitList{ l: Pos, inits: InitializerList, r: Pos },
}

#[derive(Clone, Debug)]
pub struct InitializerList {
    pub inits: Vec<Initializer>,
    pub commas: Vec<Pos>,
    pub span: Span
}

impl InitializerList {
    pub fn new() -> Self {
        Self { inits: Vec::new(), commas: Vec::new(), span: Span::default() }
    }
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    ty: Rc<Type>,
    ident: Option<Ident>,
    init: Option<Initializer>,
}

pub struct FieldDecl {
    ident: Option<Ident>, 
    colon: Option<Pos>, 
    bit_field: Option<usize>, 
    semi: Pos
}

#[derive(Debug, Clone)]
pub enum DeclKind {
    Var(VarDecl), // int a = 10;
    Field {  }, // int a : 10;
    Struct { kind: StructOrUnion, name: Option<Ident>, l: Pos, fields: Vec<Decl>, r: Pos, },
    StructRef { kind: StructOrUnion, name: Ident }, // struct name;
    Enum { kw: Span, name: Option<Ident>, l: Pos, enums: Vec<EnumField>, r: Pos,  }, // enum name { ... } 
}

#[derive(Debug, Clone)]
pub struct Decl {
    pub kind: DeclKind,
    pub ty: Option<Rc<Type>>,
    pub span: Span,
}

impl Decl {
    pub fn new(kind: DeclKind, span: Span) -> Self {
        Self { kind, ty: None, span }
    }
}

#[derive(Debug, Clone)]
pub struct DeclGroup {
    pub decls: Vec<Decl>,
    pub commas: Vec<Pos>,
    pub semi: Pos,
    pub span: Span
}

impl Default for DeclGroup {
    fn default() -> Self {
        Self {
            decls: Vec::new(),
            commas: Vec::new(),
            semi: Pos::default(),
            span: Span::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum StructOrUnionKind {
    Struct,
    Union,
}

#[derive(Debug, Clone)]
pub struct StructOrUnion {
    pub kind: StructOrUnionKind,
    pub span: Span,
}

impl StructOrUnion {
    pub fn new(token: Token) -> Self {
        let kind = match token.kind.into_keyword().unwrap() {
            Keyword::Struct => StructOrUnionKind::Struct,
            Keyword::Union => StructOrUnionKind::Union,
            _ => unreachable!()
        };
        Self { kind, span: token.span }
    }
}

// struct or union
#[derive(Debug, Clone)]
pub struct EnumField {
    pub name: Ident,
    pub eq: Option<Pos>,
    pub expr: Expr,
}