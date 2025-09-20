//!
//! 定义了语义化AST节点
//!
use crate::types::span::{Span, UnwrapSpan};
use crate::types::symbol_table::Symbol;
use enum_as_inner::EnumAsInner;
use std::rc::Rc;
use crate::types::lex::token::{Token, TokenValue};
use crate::types::lex::token_kind::TokenKind;

/// 顶层翻译单元
#[derive(Debug, Clone)]
pub struct TranslationUnit {
    pub ext_decls: Vec<ExternalDeclaration>,
}

impl UnwrapSpan for TranslationUnit {
    fn unwrap_span(&self) -> Span {
        let first = self.ext_decls.first().unwrap().unwrap_span();
        let last = self.ext_decls.last().unwrap().unwrap_span();
        first.merge(&last)
    }
}


/// 外部声明：函数或变量
#[derive(Debug, Clone)]
pub enum ExternalDeclaration {
    Function(FunctionDefinition),
    Declaration(DeclStmt),
}

impl UnwrapSpan for ExternalDeclaration {

    fn unwrap_span(&self) -> Span {
        match self {
            ExternalDeclaration::Function(x) => x.span,
            ExternalDeclaration::Declaration(x) => x.span
        }
    }
}

// 函数定义
#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub name: String,
    pub ret_ty: Box<Type>,
    pub params: Vec<Parameter>,
    pub is_variadic: bool,
    pub body: Option<Box<Block>>, // None for extern declarations
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct DeclStmt {
    pub decls: Vec<Declaration>,
    pub span: Span,
}

// 变量声明
#[derive(Debug, Clone)]
pub struct Declaration {
    pub name: String,
    pub ty: Type,
    pub storage: Option<StorageClass>,
    pub qualifiers: Qualifiers,
    pub init: Option<Initializer>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Type {
    pub kind: TypeKind,
    pub span: Span,
}

impl Type {
    pub fn new(kind: TypeKind, span: Span) -> Type {
        Type { kind, span }
    }
}
// 类型系统
#[derive(Debug, Clone)]
pub enum TypeKind {
    Void,
    Integer { signed: bool, size: IntegerSize},
    Floating { size: FloatSize},
    Pointer(Box<Type>),
    Array { elem_ty: Box<Type>, size: Option<u64>}, // size is constant-evaluated
    Function { ret_ty: Box<Type>, params: Vec<Type>, is_variadic: bool },
    Struct { name: Option<String>, fields: Vec<Field> },
    Union { name: Option<String>, fields: Vec<Field> },
    Enum { name: Option<String>, values: Vec<(String, i64)> },
    NamedType { name: String, decl_ref: Option<Rc<Symbol>> }
}

impl Type {
    pub fn string_type(len: u64, span: Span) -> Self {
        let int_kind = TypeKind::Integer { signed: false, size: IntegerSize::Char };
        let int_ty = Self {
            kind: int_kind,
            span,
        };

        let kind = TypeKind::Array { elem_ty: Box::new(int_ty), size: Some(len) };
        Self {
            kind,
            span,
        }
    }

    pub fn is_integer(&self) -> bool {
        matches!(self.kind, TypeKind::Integer { .. })
    }

    pub fn is_floating(&self) -> bool {
        matches!(self.kind, TypeKind::Floating { .. })
    }

    pub fn is_pointer(&self) -> bool {
        matches!(self.kind, TypeKind::Pointer(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self.kind, TypeKind::Array { .. })
    }

    pub fn is_function(&self) -> bool {
        matches!(self.kind, TypeKind::Function { .. })
    }

    pub fn is_arithmetic(&self) -> bool {
        self.is_integer() || self.is_floating()
    }

    pub fn is_named_type(&self) -> bool {
        matches!(self.kind, TypeKind::NamedType { .. })
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum IntegerSize {
    Char,
    Short,
    Int,
    Long,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub enum FloatSize {
    Float = 4,
    Double = 8,
}

// 存储类
#[derive(Debug, Clone)]
pub enum StorageClass {
    Typedef(Span),
    Extern(Span),
    Static(Span),
    Auto(Span),
    Register(Span),
}

// 类型限定符
#[derive(Debug, Clone)]
pub struct Qualifiers {
    pub is_const: bool,
    pub is_volatile: bool,
    // pub is_static: bool, // ?
}



// 结构体/联合体字段
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: Type,
    pub bit_width: Option<u32>, // for bitfields
    pub span: Span,
}

// 函数参数
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: Option<String>,
    pub ty: Option<Type>,
    pub span: Span,
}

// 初始化器
#[derive(Debug, Clone)]
pub enum Initializer {
    Scalar(Box<Expression>),
    List(Vec<Initializer>),
}

/// 语句块
#[derive(Debug, Clone)]
pub struct Block {
    pub items: Vec<BlockItem>,
}


#[derive(Debug, Clone)]
pub enum BlockItem {
    Declaration(DeclStmt),
    Statement(Statement),
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span
}

impl Statement {
    pub fn new(kind: StatementKind, span: Span) -> Self {
        Self { kind, span }
    }
}

// 语句
/// todo 差别太大 IF For
#[derive(Debug, Clone)]
pub enum StatementKind {
    Labeled {
        label: String,
        stmt: Box<Statement>,
    },
    Case { // constant-evaluated
        value: i64,
        stmt: Box<Statement>,
    },
    Default {
        stmt: Box<Statement>,
    },
    Block(Box<Block>),
    Expression(Option<Box<Expression>>),
    If {
        cond: Box<Expression>,
        then_stmt: Box<Statement>,
        else_stmt: Option<Box<Statement>>,
    },
    Switch {
        cond: Box<Expression>,
        body: Box<Statement>,
    },
    While {
        cond: Box<Expression>,
        body: Box<Statement>,
    },
    DoWhile {
        body: Box<Statement>,
        cond: Box<Expression>,
    },
    For {
        init: Option<Box<Expression>>,
        cond: Option<Box<Expression>>,
        step: Option<Box<Expression>>,
        body: Box<Statement>,
    },
    Goto {
        label: String,
    },
    Continue,
    Break,
    Return(Option<Box<Expression>>),
}
// 表达式
#[derive(Debug, Clone)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub ty: Option<Type>, // 类型交给后期多次遍历时填充，
    pub span: Span,
}

impl Expression {

    pub fn new(kind: ExpressionKind, ty: Option<Type>, span: Span) -> Expression {
        Expression { kind, ty, span }
    }

    pub fn is_lvalue(&self) -> bool {
        match &self.kind {
            ExpressionKind::Id { .. } => true,                    // 变量
            ExpressionKind::ArrayAccess { .. } => true,             // a[i]
            ExpressionKind::FieldAccess { .. } => true,            // s.f 或 s->f
            ExpressionKind::Arrow { .. } => true,
            ExpressionKind::PostInc(_) => true,
            ExpressionKind::PostDec(_) => true,
            ExpressionKind::Unary { op, .. } => op.is_lvalue(), // *p
            ExpressionKind::Cast { expr, .. } => expr.is_lvalue(),
            ExpressionKind::Assign { .. } => true,                // a = b 是右值
            _ => false,
        }
    }

    pub fn is_rvalue(&self) -> bool {
        !self.is_lvalue()
    }

}

#[derive(Debug, Clone, EnumAsInner)]
pub enum ExpressionKind {
    Literal(Constant, Span),
    Id { // decl_ref 指向符号表索引
        name: String,
        decl_ref: Option<Rc<Symbol>>
    },
    ArrayAccess {
        base: Box<Expression>,
        index: Box<Expression>
    },
    Call {
        func: Box<Expression>,
        args: Vec<Expression>
    },
    FieldAccess {
        base: Box<Expression>,
        field: String
    },
    Arrow {
        base: Box<Expression>,
        field: String
    },
    PostInc(Box<Expression>),
    PostDec(Box<Expression>),
    PreInc(Box<Expression>),
    PreDec(Box<Expression>),
    Unary {
        op: UnaryOp,
        expr: Box<Expression>
    },
    SizeofExpr(Box<Expression>),
    SizeofType(Box<Type>),
    Cast {
        ty: Box<Type>,
        expr: Box<Expression>
    },
    Binary {
        op: BinaryOp,
        lhs: Box<Expression>,
        rhs: Box<Expression>
    },
    Conditional {
        cond: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>
    },
    Assign {
        lhs: Box<Expression>,
        op: AssignOp,
        rhs: Box<Expression>
    },
    Comma {
        exprs: Vec<Expression>
    },
}

#[derive(Debug, Clone)]
pub struct Constant {
    pub kind: ConstantKind,
    pub span: Span,
}

impl Constant {
    pub fn new(kind: ConstantKind, span: Span) -> Constant {
        Constant { kind, span }
    }

    pub fn get_type(&self) -> Type {
        let span = self.span;
        let kind = match &self.kind {
            ConstantKind::Int(_) => TypeKind::Integer { signed: true, size: IntegerSize::Int },
            ConstantKind::Float(_) => TypeKind::Floating { size: FloatSize::Float },
            ConstantKind::Char(_) => TypeKind::Integer { signed: true, size: IntegerSize::Char },
            ConstantKind::String(x) => {
                let int_kind = TypeKind::Integer { signed: false, size: IntegerSize::Char };
                let int_ty = Type::new(int_kind, span);
                TypeKind::Array { elem_ty: Box::new(int_ty), size: Some(x.len() as u64) }
            },
        };

        Type::new(kind, span)
    }
}

impl TryFrom<Token> for Constant {
    type Error = &'static str;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        let span = value.span;
        let kind: ConstantKind = value.try_into()?;
        Ok(Self::new(kind, span))
    }
}

#[derive(Debug, Clone)]
pub enum ConstantKind {
    Int(i64),
    Float(f64),
    Char(u8),
    String(String), // 合并后的字符串字面量
}

impl TryFrom<Token> for ConstantKind {
    type Error = &'static str;
    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value.value {
            TokenValue::Number { value, .. } => Ok(Self::Int(value as i64)),
            TokenValue::Float(x) => Ok(Self::Float(x)),
            TokenValue::String(x) => Ok(Self::String(x)),
            TokenValue::Char(x) => Ok(Self::Char(x)),
            TokenValue::Other => Err("Failed to convert Token to ConstantKind")
        }
    }
}

#[derive(Debug, Clone)]
pub enum UnaryOpKind {
    AddressOf,
    Deref,
    Plus,
    Minus,
    BitNot,
    LogicalNot,
}

#[derive(Debug, Clone)]
pub struct UnaryOp {
    pub kind: UnaryOpKind,
    pub span: Span,
}

impl UnaryOp {
    pub fn new(kind: UnaryOpKind, span: Span) -> UnaryOp {
        Self {
            kind,
            span
        }
    }

    pub fn is_lvalue(&self) -> bool {
        match self.kind {
            UnaryOpKind::AddressOf => false,
            UnaryOpKind::Deref => true,
            UnaryOpKind::Plus => false,
            UnaryOpKind::Minus => false,
            UnaryOpKind::BitNot => true,
            UnaryOpKind::LogicalNot => false
        }
    }
}

#[derive(Debug, Clone)]
pub enum BinaryOpKind {
    Add, Sub, Mul, Div, Mod,
    Shl, Shr,
    Lt, Gt, Le, Ge, Eq, Ne,
    BitAnd, BitXor, BitOr,
    LogicalAnd, LogicalOr,
}

impl TryFrom<Token> for BinaryOpKind {
    type Error = &'static str; // todo 后期可以用this error

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value.kind {
            TokenKind::OpPlus => Ok(BinaryOpKind::Add),
            TokenKind::OpMinus => Ok(BinaryOpKind::Sub),
            TokenKind::OpTimes => Ok(BinaryOpKind::Mul),
            TokenKind::OpDivide => Ok(BinaryOpKind::Div),
            TokenKind::OpMod => Ok(BinaryOpKind::Mod),
            TokenKind::OpLShift => Ok(BinaryOpKind::Shl),
            TokenKind::OpRShift => Ok(BinaryOpKind::Shr),
            TokenKind::OpLt => Ok(BinaryOpKind::Lt),
            TokenKind::OpGt => Ok(BinaryOpKind::Gt),
            TokenKind::OpLe => Ok(BinaryOpKind::Le),
            TokenKind::OpGe => Ok(BinaryOpKind::Ge),
            TokenKind::OpEq => Ok(BinaryOpKind::Eq),
            TokenKind::OpNe => Ok(BinaryOpKind::Ne),
            TokenKind::OpBitand =>Ok(BinaryOpKind::BitAnd),
            TokenKind::OpXor => Ok(BinaryOpKind::BitXor),
            TokenKind::OpBitor => Ok(BinaryOpKind::BitOr),
            TokenKind::OpAnd => Ok(BinaryOpKind::LogicalAnd),
            TokenKind::OpOr => Ok(BinaryOpKind::LogicalOr),
            _ => Err("Failed to convert binary operator"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BinaryOp {
    pub kind: BinaryOpKind,
    pub span: Span
}

impl BinaryOp {
    pub fn new(kind: BinaryOpKind, span: Span) -> BinaryOp {
        Self { kind, span }
    }
}

#[derive(Debug, Clone)]
pub enum AssignOpKind {
    Assign,
    MulAssign,
    DivAssign,
    ModAssign,
    AddAssign,
    SubAssign,
    ShlAssign,
    ShrAssign,
    AndAssign,
    XorAssign,
    OrAssign,
}

impl TryFrom<Token> for AssignOpKind {
    type Error = &'static str;
    fn try_from(value: Token) -> Result<Self, Self::Error> {
       match value.kind {
            TokenKind::OpAssign => Ok(AssignOpKind::Assign),
            TokenKind::OpMulAssign => Ok(AssignOpKind::MulAssign),
            TokenKind::OpDivAssign => Ok(AssignOpKind::DivAssign),
            TokenKind::OpModAssign => Ok(AssignOpKind::ModAssign),
            TokenKind::OpAddAssign => Ok(AssignOpKind::AddAssign),
            TokenKind::OpSubAssign => Ok(AssignOpKind::SubAssign),
            TokenKind::OpLShiftAssign => Ok(AssignOpKind::ShlAssign),
            TokenKind::OpRShiftAssign => Ok(AssignOpKind::ShrAssign),
            TokenKind::OpAndAssign => Ok(AssignOpKind::AndAssign),
            TokenKind::OpOrAssign => Ok(AssignOpKind::OrAssign),
            _ => Err("Failed to convert assignment operator"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AssignOp {
    pub kind: AssignOpKind,
    pub span: Span,
}

impl AssignOp {
    pub fn new(kind: AssignOpKind, span: Span) -> AssignOp {
        Self { kind, span }
    }
}