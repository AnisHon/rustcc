use std::rc::Rc;
use enum_as_inner::EnumAsInner;
use crate::lex::types::token::Token;
use crate::lex::types::token_kind::Keyword;
use crate::parser::types::ast::expr::Expr;
use crate::parser::types::common::Ident;
use crate::parser::types::sema::decl::decl_context::{DeclContext, DeclContextRef, EnumDeclContext, RecordDeclContext};
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


#[derive(Debug, Clone, EnumAsInner)]
pub enum DeclKind {
    VarDecl { name: Option<Ident> },
    VarInit { var: Rc<Decl>, eq: Option<Pos>, init: Option<Initializer>, }, // int a = 10;
    RecordField { var: Rc<Decl>, colon: Option<Pos>, bit_field: Option<Box<Expr>> }, // int a : 10;
    Record { kind: StructOrUnion, name: Option<Ident>, l: Pos, fields: Vec<DeclGroup>, r: Pos, decl_context: RecordDeclContext },
    RecordRef { kind: StructOrUnion, name: Ident }, // struct name;
    EnumField { name: Ident, eq: Option<Pos>, expr: Option<Box<Expr>> },
    Enum { kw: Span, name: Option<Ident>, l: Pos, enums: EnumFieldList, r: Pos, decl_context: EnumDeclContext }, // enum name { ... } 
    EnumRef { kw: Span, name: Ident },
}

#[derive(Debug, Clone)]
pub struct Decl {
    pub kind: DeclKind,
    pub ty: Option<Rc<Type>>,
    pub span: Span,
}

impl Decl {
    pub fn new(kind: DeclKind, ty: Rc<Type>, span: Span) -> Self {
        Self { kind, ty: Some(ty), span }
    }
    
    pub fn new_rc(kind: DeclKind, ty: Rc<Type>, span: Span) -> Rc<Self> {
        Rc::new(Self::new(kind, ty, span))
    }
    
    pub fn get_name(&self) -> Option<&Ident> {
        use DeclKind::*;
        match &self.kind {
            VarDecl { name }
            | Record { name, .. }
            | Enum { name, .. } => name.as_ref(),
            RecordRef { name, .. }
            | EnumRef { name, .. } 
            | EnumField { name, .. } => Some(name), 
            VarInit { var, .. }
            | RecordField { var, .. } => var.get_name(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeclGroup {
    pub decls: Vec<Rc<Decl>>,
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
    pub expr: Option<Box<Expr>>,
    pub span: Span,
}


#[derive(Clone, Debug)]
pub struct EnumFieldList {
    pub decls: Vec<EnumField>,
    pub commas: Vec<Pos>,
    pub span: Span
}

impl Default for EnumFieldList {
    fn default() -> Self {
        Self {
            decls: Vec::new(),
            commas: Vec::new(),
            span: Span::default(),
        }
    }
}
