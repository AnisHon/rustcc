use std::cell::RefCell;
use crate::lex::types::token::Token;
use crate::lex::types::token_kind::Keyword;
use crate::parser::semantic::ast::expr::Expr;
use crate::parser::semantic::common::Ident;
use crate::parser::semantic::decl_spec::StorageSpec;
use crate::parser::semantic::sema::decl::decl_context::DeclContextRef;
use crate::parser::semantic::sema::sema_type::Type;
use crate::types::span::{Pos, Span};
use enum_as_inner::EnumAsInner;
use std::rc::Rc;
use crate::parser::semantic::ast::stmt::Stmt;

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
    TypeDef,
    ParamVar,
    FuncRef, // 函数声明
    VarInit {  // int a = 10;
        eq: Option<Pos>, 
        init: Option<Initializer>, 
    },
    Func { // 函数定义
        body: Box<Stmt>,
    },
    RecordField {  // int a : 10;
        colon: Option<Pos>, 
        bit_field: Option<Box<Expr>> 
    },
    Record { 
        kind: StructOrUnion, 
        l: Pos,
        fields: Vec<DeclGroup>,
        r: Pos,
        decl_context: DeclContextRef
    },
    RecordRef { 
        kind: StructOrUnion, 
    }, // struct name;
    EnumField { 
        eq: Option<Pos>, 
        expr: Option<Box<Expr>>
    },
    Enum { // enum name { ... } 
        kw: Span, 
        l: Pos,
        enums: Vec<DeclRef>,
        commas: Vec<Pos>, 
        r: Pos,
        decl_context: DeclContextRef
    }, 
    EnumRef { kw: Span },
}

pub type DeclRef = Rc<RefCell<Decl>>;


#[derive(Debug, Clone)]
pub struct Decl {
    pub storage: Option<StorageSpec>,
    pub name: Option<Ident>,
    pub kind: DeclKind,
    pub ty: Rc<Type>,
    pub span: Span,
}

impl Decl {
    pub fn get_name(&self) -> Option<&Ident> {
        self.name.as_ref()
    }

    pub fn new_ref(decl: Self) -> DeclRef {
        Rc::new(RefCell::new(decl))
    }
}

#[derive(Debug, Clone)]
pub struct DeclGroup {
    pub decls: Vec<DeclRef>,
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
