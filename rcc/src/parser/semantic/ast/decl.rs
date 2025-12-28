use crate::parser::ast::common::StructOrUnion;
use crate::parser::ast::{DeclKey, ExprKey, TypeKey};
use crate::parser::semantic::ast::stmt::Stmt;
use crate::parser::semantic::common::Ident;
use crate::parser::semantic::decl_spec::StorageSpec;
use crate::types::span::{Pos, Span};
use enum_as_inner::EnumAsInner;

#[derive(Debug, Clone)]
pub enum Initializer {
    Expr(ExprKey),
    InitList {
        l: Pos,
        inits: InitializerList,
        r: Pos,
    },
}

#[derive(Clone, Debug)]
pub struct InitializerList {
    pub inits: Vec<Initializer>,
    pub commas: Vec<Pos>,
    pub span: Span,
}

impl InitializerList {
    pub fn new() -> Self {
        Self {
            inits: Vec::new(),
            commas: Vec::new(),
            span: Span::default(),
        }
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
    VarInit {
        // int a = 10;
        init: Option<Initializer>,
    },
    Func {
        // 函数定义
        body: Box<Stmt>,
    },
    RecordField {
        // int a : 10;
        bit_field: Option<ExprKey>,
    },
    Record {
        kind: StructOrUnion,
        fields: Option<Vec<DeclGroup>>, // 当 fields 为 none 时为不完全类型
                                        // decl_context: DeclContextRef,
    },
    EnumField {
        expr: Option<ExprKey>,
    },
    Enum {
        enums: Option<Vec<DeclKey>>, // 当 enums 为 none 时为不完全类型
                                     // decl_context: DeclContextRef,
    },
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
    pub span: Span,
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
