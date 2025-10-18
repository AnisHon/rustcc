use crate::lex::types::token::Token;
use crate::lex::types::token_kind::Keyword;
use crate::parser::types::ast::expr::Expr;
use crate::parser::types::common::Ident;
use crate::parser::types::decl_spec::StorageSpec;
use crate::parser::types::sema::decl::decl_context::DeclContextRef;
use crate::parser::types::sema::sema_type::Type;
use crate::types::span::{Pos, Span};
use enum_as_inner::EnumAsInner;
use std::rc::Rc;

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
    VarInit { eq: Option<Pos>, init: Option<Initializer>, }, // int a = 10;
    RecordField { colon: Option<Pos>, bit_field: Option<Box<Expr>> }, // int a : 10;
    Record { kind: StructOrUnion, name: Option<Ident>, l: Pos, fields: Vec<DeclGroup>, r: Pos, decl_context: DeclContextRef },
    RecordRef { kind: StructOrUnion, name: Ident }, // struct name;
    EnumField { eq: Option<Pos>, expr: Option<Box<Expr>> },
    Enum { kw: Span, l: Pos, enums: EnumFieldList, r: Pos, decl_context: DeclContextRef }, // enum name { ... }
    EnumRef { kw: Span },
}

#[derive(Debug, Clone)]
pub struct Decl {
    pub storage: Option<StorageSpec>,
    pub name: Option<Ident>,
    pub kind: DeclKind,
    pub ty: Option<Rc<Type>>,
    pub span: Span,
}

impl Decl {
    pub fn new(kind: DeclKind, name: Option<Ident>, span: Span) -> Self {
        Self { storage: None, name: None, kind, ty: None, span }
    }
    
    pub fn new_rc(kind: DeclKind, name: Option<Ident>, span: Span) -> Rc<Self> {
        Rc::new(Self::new(kind, name, span))
    }
    
    pub fn get_name(&self) -> Option<&Ident> {
        self.name.as_ref()
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
