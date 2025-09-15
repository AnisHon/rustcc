//!
//! 定义了语义化AST节点
//!
use crate::types::span::{Span, UnwrapSpan};
use crate::types::symbol_table::Symbol;
use enum_as_inner::EnumAsInner;
use std::rc::Rc;



// 顶层翻译单元
#[derive(Debug, Clone)]
pub struct TranslationUnit {
    pub ext_decls: Vec<ExternalDeclaration>,
    pub span: Span,
}


// 外部声明：函数或变量
#[derive(Debug, Clone)]
pub enum ExternalDeclaration {
    Function(Box<FunctionDefinition>, Span),
    Declaration(Box<DeclStmt>, Span),
}

impl UnwrapSpan for ExternalDeclaration {

    fn unwrap_span(&self) -> Span {
        match self {
            ExternalDeclaration::Function(_, x) => *x,
            ExternalDeclaration::Declaration(_, x) => *x
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
    pub ty: Box<Type>,
    pub storage: Option<StorageClass>,
    pub qualifiers: Qualifiers,
    pub init: Option<Box<Initializer>>,
    pub span: Span,
}


// 类型系统
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Void(Span),
    Integer { signed: bool, size: IntegerSize, span: Span},
    Floating { size: FloatSize, span: Span},
    Pointer(Box<Type>, Span),
    Array { elem_ty: Box<Type>, size: Option<u64>, span: Span}, // size is constant-evaluated
    Function { ret_ty: Box<Type>, params: Vec<Type>, is_variadic: bool, span: Span },
    Struct { name: Option<String>, fields: Vec<Field>, span: Span },
    Union { name: Option<String>, fields: Vec<Field>, span: Span },
    Enum { name: Option<String>, values: Vec<(String, i64)>, span: Span },
    NamedType { name: String, decl_ref: Option<Rc<Symbol>>,span: Span, }
}

impl UnwrapSpan for Type {
    fn unwrap_span(&self) -> Span {
        match self {
            Type::Void(x) => *x,
            Type::Integer { span, .. } => *span,
            Type::Floating { span, .. } => *span,
            Type::Pointer(_, x) => *x,
            Type::Array { span, .. } => *span,
            Type::Function { span, .. } => *span,
            Type::Struct { span, .. } => *span,
            Type::Union { span, .. } => *span,
            Type::Enum { span, .. } => *span,
            Type::NamedType { span, .. } => *span,
        }
    }
}

impl Type {
    pub fn string_type(len: u64, span: Span) -> Type {
        Type::Array { elem_ty: Box::new(Type::Integer { signed: false, size: IntegerSize::Char, span }), size: Some(len), span }
    }

    /// 返回类型等级（越大精度越高）
    pub fn rank(&self) -> u8 {
        match self {
            Type::Integer { size, .. } => match size {
                IntegerSize::Char => 1,
                IntegerSize::Short => 2,
                IntegerSize::Int => 3,
                IntegerSize::Long => 4,
            },
            Type::Floating { size, .. } => match size {
                FloatSize::Float => 6,
                FloatSize::Double => 7,
            },
            Type::Pointer(_, _) => 9, // 指针类型等级最高，用于 +,- 可合法
            _ => 0,
        }
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, Type::Integer { .. })
    }

    pub fn is_floating(&self) -> bool {
        matches!(self, Type::Floating { .. })
    }

    pub fn is_pointer(&self) -> bool {
        matches!(self, Type::Pointer(_, _))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Type::Array { .. })
    }

    pub fn is_function(&self) -> bool {
        matches!(self, Type::Function { .. })
    }

    pub fn is_arithmetic(&self) -> bool {
        self.is_integer() || self.is_floating()
    }

    pub fn is_named_type(&self) -> bool {
        matches!(self, Type::NamedType { .. })
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
#[derive(PartialEq)]
pub struct Field {
    pub name: String,
    pub ty: Box<Type>,
    pub bit_width: Option<u32>, // for bitfields
    pub span: Span,
}

// 函数参数
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: Option<String>,
    pub ty: Option<Box<Type>>,
    pub span: Span,
}

// 初始化器
#[derive(Debug, Clone)]
pub enum Initializer {
    Scalar(Box<Expression>, Span),
    List(Vec<Initializer>, Span),
}

/// 语句块
#[derive(Debug, Clone)]
pub struct Block {
    pub items: Vec<BlockItem>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum BlockItem {
    Declaration(Box<DeclStmt>, Span),
    Statement(Box<Statement>, Span),
}

// 语句
/// todo 差别太大 IF For
#[derive(Debug, Clone)]
pub enum Statement {
    Labeled {
        label: String,
        stmt: Box<Statement>,
        span: Span
    },
    Case { // constant-evaluated
        value: i64,
        stmt: Box<Statement>,
        span: Span
    },
    Default {
        stmt: Box<Statement>,
        span: Span
    },
    Block(Box<Block>, Span),
    Expression(Option<Box<Expression>>, Span),
    If {
        cond: Box<Expression>,
        then_stmt: Box<Statement>,
        else_stmt: Option<Box<Statement>>,
        span: Span
    },
    Switch {
        cond: Box<Expression>,
        body: Box<Statement>,
        span: Span
    },
    While {
        cond: Box<Expression>,
        body: Box<Statement>,
        span: Span
    },
    DoWhile {
        body: Box<Statement>,
        cond: Box<Expression>,
        span: Span
    },
    For {
        init: Option<Box<Expression>>,
        cond: Option<Box<Expression>>,
        step: Option<Box<Expression>>,
        body: Box<Statement>,
        span: Span
    },
    Goto {
        label: String,
        span: Span
    },
    Continue(Span),
    Break(Span),
    Return(Option<Box<Expression>>, Span),
}

impl Statement {

    pub fn unwrap_span(&self) -> Span {
        match self {
            Statement::Labeled { span, .. }
            | Statement::Case { span, .. }
            | Statement::Default { span, .. }
            | Statement::Block(_, span)
            | Statement::Expression(_, span)
            | Statement::If { span, .. }
            | Statement::Switch { span, .. }
            | Statement::While { span, .. }
            | Statement::DoWhile { span, .. }
            | Statement::For { span, .. }
            | Statement::Goto { span, .. }
            | Statement::Continue(span)
            | Statement::Break(span)
            | Statement::Return(_, span) => *span,
            // Statement::Decl (decl) => decl.span,
        }
    }


}

// 表达式
#[derive(Debug, Clone)]
pub struct Expression {
    pub kind: Box<ExpressionKind>,
    pub ty: Option<Box<Type>>, // 类型交给后期多次遍历时填充，
    pub span: Span,
}

impl Expression {

    pub fn is_lvalue(&self) -> bool {
        match self.kind.as_ref() {
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
pub enum Constant {
    Int(i64, Span),
    Float(f64, Span),
    Char(u8, Span),
    String(String, Span), // 合并后的字符串字面量
}


#[derive(Debug, Clone)]
pub enum UnaryOp {
    AddressOf(Span),
    Deref(Span),
    Plus(Span),
    Minus(Span),
    BitNot(Span),
    LogicalNot(Span),
}

impl UnaryOp {
    pub fn unwrap_span(&self) -> Span {
        match self {
            UnaryOp::AddressOf(x) => *x,
            UnaryOp::Deref(x) => *x,
            UnaryOp::Plus(x) => *x,
            UnaryOp::Minus(x) => *x,
            UnaryOp::BitNot(x) => *x,
            UnaryOp::LogicalNot(x) => *x,
        }
    }

    pub fn is_lvalue(&self) -> bool {
        match self {
            UnaryOp::AddressOf(_) => false,
            UnaryOp::Deref(_) => true,
            UnaryOp::Plus(_) => false,
            UnaryOp::Minus(_) => false,
            UnaryOp::BitNot(_) => true,
            UnaryOp::LogicalNot(_) => false
        }
    }
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add(Span),
    Sub(Span),
    Mul(Span),
    Div(Span),
    Mod(Span),
    Shl(Span),
    Shr(Span),
    Lt(Span),
    Gt(Span),
    Le(Span),
    Ge(Span),
    Eq(Span),
    Ne(Span),
    BitAnd(Span),
    BitXor(Span),
    BitOr(Span),
    LogicalAnd(Span),
    LogicalOr(Span),
}

impl BinaryOp {
    pub fn unwrap_span(&self) -> Span {
        match self {
            BinaryOp::Add(s) => *s,
            BinaryOp::Sub(s) => *s,
            BinaryOp::Mul(s) => *s,
            BinaryOp::Div(s) => *s,
            BinaryOp::Mod(s) => *s,
            BinaryOp::Shl(s) => *s,
            BinaryOp::Shr(s) => *s,
            BinaryOp::Lt(s) => *s,
            BinaryOp::Gt(s) => *s,
            BinaryOp::Le(s) => *s,
            BinaryOp::Ge(s) => *s,
            BinaryOp::Eq(s) => *s,
            BinaryOp::Ne(s) => *s,
            BinaryOp::BitAnd(s) => *s,
            BinaryOp::BitXor(s) => *s,
            BinaryOp::BitOr(s) => *s,
            BinaryOp::LogicalAnd(s) => *s,
            BinaryOp::LogicalOr(s) => *s,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AssignOp {
    Assign(Span),
    MulAssign(Span),
    DivAssign(Span),
    ModAssign(Span),
    AddAssign(Span),
    SubAssign(Span),
    ShlAssign(Span),
    ShrAssign(Span),
    AndAssign(Span),
    XorAssign(Span),
    OrAssign(Span),
}
