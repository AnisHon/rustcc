use crate::lex::types::token::Token;
use crate::lex::types::token_kind::Keyword;
use crate::parser::semantic::ast::stmt::Stmt;
use crate::parser::semantic::common::Ident;
use crate::parser::semantic::decl_spec::StorageSpec;
use crate::parser::semantic::sema::decl::decl_context::DeclContextRef;
use crate::types::span::{Pos, Span};
use enum_as_inner::EnumAsInner;
use slotmap::new_key_type;
use crate::parser::ast::exprs::ExprKey;
use crate::parser::ast::types::TypeKey;

new_key_type! {
    pub struct DeclKey;
}

#[derive(Debug, Clone)]
pub enum Initializer {
    Expr(ExprKey),
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
pub struct Decl {
    pub storage: Option<StorageSpec>,
    pub name: Option<Ident>,
    pub kind: DeclKind,
    pub ty: TypeKey,
    pub span: Span,
}

#[derive(Debug, Clone, EnumAsInner)]
pub enum DeclKind {
    TypeDef,
    ParamVar,
    FuncRef, // 函数声明
    VarInit {  // int a = 10;
        init: Option<Initializer>, 
    },
    Func { // 函数定义
        body: Box<Stmt>,
    },
    RecordField {  // int a : 10;
        bit_field: Option<u128> 
    },
    Record { 
        kind: StructOrUnion, 
        fields: Vec<DeclGroup>,
        decl_context: DeclContextRef
    },
    RecordRef { 
        kind: StructOrUnion, 
    }, // struct name;
    EnumField { 
        expr: Option<u128>
    },
    Enum { // enum name { ... } 
        kw: Span, 
        enums: Vec<DeclKey>,
        decl_context: DeclContextRef
    }, 
    EnumRef { kw: Span },
}



impl Decl {
    pub fn get_name(&self) -> Option<&Ident> {
        self.name.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct DeclGroup {
    pub decls: Vec<DeclKey>,
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
