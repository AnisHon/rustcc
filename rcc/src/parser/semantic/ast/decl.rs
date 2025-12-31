use crate::parser::ast::common::StructOrUnion;
use crate::parser::ast::{DeclKey, ExprKey, TypeKey};
use crate::parser::semantic::ast::stmt::Stmt;
use crate::parser::semantic::common::Ident;
use crate::parser::semantic::decl_spec::{FuncSpec, StorageSpec};
use crate::types::span::{Pos, Span};
use enum_as_inner::EnumAsInner;

#[derive(Debug, Clone)]
pub enum Initializer {
    Expr(ExprKey),
    InitList { inits: InitializerList },
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

    // 变量声明，变量定义
    VarDecl {
        def: Option<DeclKey>,
    },
    VarDef {
        init: Option<Initializer>,
    },

    // 函数声明 函数定义
    FuncDecl {
        def: Option<DeclKey>,
    },
    FuncDef {
        inline: Option<FuncSpec>,
        body: Box<Stmt>,
    },

    // Record 成员 声明 定义
    RecordField {
        // int a : 10;
        bit_field: Option<ExprKey>,
    },
    RecordDecl {
        kind: StructOrUnion,
        def: Option<DeclKey>,
    },
    RecordDef {
        kind: StructOrUnion,
        fields: Vec<DeclGroup>, // 当 fields 为 none 时为不完全类型
    },

    // enum 成员 声明 定义
    EnumField {
        expr: Option<ExprKey>,
    },
    EnumDecl {
        def: Option<DeclKey>,
    },
    EnumDef {
        enums: Option<Vec<DeclKey>>, // 当 enums 为 none 时为不完全类型
    },
}

impl Decl {
    pub fn get_name(&self) -> Option<&Ident> {
        self.name.as_ref()
    }

    /// 是否是声明
    pub fn is_decl(&self) -> bool {
        use DeclKind::*;
        matches!(
            &self.kind,
            FuncDecl { .. } | RecordDecl { .. } | EnumDecl { .. } | VarDecl { .. }
        )
    }

    /// 是否是定义
    pub fn is_def(&self) -> bool {
        use DeclKind::*;
        matches!(
            &self.kind,
            FuncDef { .. } | RecordDef { .. } | EnumDef { .. } | VarDef { .. }
        )
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
