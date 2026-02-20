use crate::parser::ast::types::QualType;
use crate::types::parser::ast::common::StructOrUnion;
use crate::types::parser::ast::decls::initializer::Initializer;
use crate::types::parser::ast::stmt::Stmt;
use crate::types::parser::ast::{DeclKey, ExprKey};
use crate::types::parser::common::Ident;
use crate::types::parser::decl_spec::{FuncSpec, StorageSpec};
use crate::types::span::Span;
use enum_as_inner::EnumAsInner;

/// 声明状态，大致上表示 声明 和 定义，具体情况具体分析, typedef 永远认为是 Incomplete
pub enum DeclStatus {
    Complete,   // Complete definition
    Incomplete, // Forward/incomplete declaration
}

/// 声明类型
pub enum DeclType {
    Typedef, // typedef
    Object,  // variable function
    Tag,     // enum struct union
}

/// AST Declaration 对应的声明节点
/// - `name`: 声明对象的名字
/// - `kind`: 声明对象内容
/// - `ty`: 声明对象的C类型
/// - `span`: 位置区间
#[derive(Debug, Clone)]
pub struct Decl {
    pub name: Option<Ident>,
    pub kind: DeclKind,
    pub ty: QualType,
    pub storage: Option<StorageSpec>,
    pub span: Span,
}

impl Decl {
    pub fn new(
        name: Option<Ident>,
        kind: DeclKind,
        ty: QualType,
        storage: Option<StorageSpec>,
        span: Span,
    ) -> Decl {
        if let Some(storage) = storage.as_ref() {
            debug_assert_eq!(
                kind.is_type_def(),
                storage.kind.is_typedef(),
                "typedef不一致"
            )
        }
        Decl {
            name,
            kind,
            ty,
            storage,
            span,
        }
    }
    pub fn get_name(&self) -> Option<&Ident> {
        self.name.as_ref()
    }

    /// 声明状态
    pub fn get_status(&self) -> DeclStatus {
        match &self.kind {
            DeclKind::TypeDef => DeclStatus::Complete,
            DeclKind::Var(x) => x.get_status(),
            DeclKind::Func(x) => x.get_status(),
            DeclKind::Record(x) => x.get_status(),
            DeclKind::Enum(x) => x.get_status(),
            DeclKind::EnumField(_) => DeclStatus::Complete,
            DeclKind::RecordField(_) => unreachable!("对 record 成员 取声明状态没有意义"),
        }
    }

    /// 声明种类
    pub fn decl_type(&self) -> DeclType {
        match &self.kind {
            DeclKind::TypeDef => DeclType::Typedef,
            DeclKind::Var(_) => DeclType::Object,
            DeclKind::Func(_) => DeclType::Object,
            DeclKind::Record(_) => DeclType::Tag,
            DeclKind::EnumField(_) => DeclType::Object,
            DeclKind::RecordField(_) => unreachable!("对Record成员取 声明类型无意义"),
            DeclKind::Enum(_) => DeclType::Tag,
        }
    }
}

/// 变量声明定义，不会检查是否存在 `is_def = false`, 但 `init` 存在的情况
/// 由于变量的特殊性，存在暂定定义
/// # Contents
/// - `is_def`: 是否是定义
/// - `init`: 初始化
#[derive(Debug, Clone)]
pub struct VarDecl {
    pub is_def: bool, // 是否是定义
    pub init: Option<Initializer>,
}

impl VarDecl {
    pub fn new(is_def: bool, init: Option<Initializer>) -> Self {
        Self { is_def, init }
    }

    pub fn get_status(&self) -> DeclStatus {
        if self.is_def {
            DeclStatus::Incomplete
        } else {
            DeclStatus::Complete
        }
    }
}

/// 函数声明定义
/// 如果body为空则为定义，如果body存在则为定义
#[derive(Debug, Clone)]
pub struct FuncDecl {
    pub inline: Option<FuncSpec>,
    pub body: Option<Box<Stmt>>,
}

impl FuncDecl {
    pub fn new(inline: Option<FuncSpec>, body: Option<Box<Stmt>>) -> Self {
        if let Some(body) = body.as_ref() {
            debug_assert!(body.kind.is_compound(), "函数定义的body必须是复合语句")
        }
        Self { inline, body }
    }

    pub fn get_status(&self) -> DeclStatus {
        match self.body.as_ref() {
            None => DeclStatus::Incomplete,
            Some(_) => DeclStatus::Complete,
        }
    }
}

/// struct union 的声明和定义
#[derive(Debug, Clone)]
pub struct RecordDecl {
    pub kind: StructOrUnion,
    pub fields: Option<Vec<DeclGroup>>, // 当 fields 为 none 时为不完全类型
}

impl RecordDecl {
    pub fn new(kind: StructOrUnion, fields: Option<Vec<DeclGroup>>) -> Self {
        Self { kind, fields }
    }

    pub fn get_status(&self) -> DeclStatus {
        match self.fields.as_ref() {
            None => DeclStatus::Incomplete,
            Some(_) => DeclStatus::Complete,
        }
    }
}

/// struct 成员声明
#[derive(Debug, Clone)]
pub struct RecordFieldDecl {
    pub name: Option<Ident>,
    pub bit_field: Option<ExprKey>,
}

impl RecordFieldDecl {
    pub fn new(name: Option<Ident>, bit_field: Option<ExprKey>) -> Self {
        Self { name, bit_field }
    }
}

/// enum 声明或定义
#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub enums: Option<Vec<DeclKey>>, // 当 enums 为 none 时为不完全类型
}

impl EnumDecl {
    pub fn new(enums: Option<Vec<DeclKey>>) -> Self {
        Self { enums }
    }

    pub fn get_status(&self) -> DeclStatus {
        match self.enums.as_ref() {
            None => DeclStatus::Incomplete,
            Some(_) => DeclStatus::Complete,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnumFieldDecl {
    pub expr: Option<ExprKey>,
}

impl EnumFieldDecl {
    pub fn new(expr: Option<ExprKey>) -> Self {
        Self { expr }
    }
}

#[derive(Debug, Clone, EnumAsInner)]
pub enum DeclKind {
    TypeDef,

    // 变量声明，变量定义
    Var(VarDecl),

    // 函数声明 函数定义
    Func(FuncDecl),

    // Record 成员 声明 定义
    Record(RecordDecl),
    RecordField(RecordFieldDecl),

    // enum 成员 声明 定义
    EnumField(EnumFieldDecl),
    Enum(EnumDecl),
}

/// 声明组
#[derive(Debug, Clone)]
pub struct DeclGroup {
    pub decls: Vec<DeclKey>,
    // pub commas: Vec<Pos>,
    // pub semi: Pos,
    pub span: Span,
}

impl Default for DeclGroup {
    fn default() -> Self {
        Self {
            decls: Vec::new(),
            // commas: Vec::new(),
            // semi: Pos::default(),
            span: Span::default(),
        }
    }
}
