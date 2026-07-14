use super::{CType, DeclId, Initializer};
use crate::lex::token::StringEncoding;
use crate::source::SourceRange;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueCategory {
    LValue,
    RValue,
    Function,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub ty: CType,
    pub category: ValueCategory,
    pub range: SourceRange,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    Integer(i128),
    Floating(f64),
    Character {
        value: i64,
        encoding: StringEncoding,
    },
    String {
        value: String,
        encoding: StringEncoding,
    },
    Identifier {
        name: String,
        declaration: DeclId,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expression>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Assignment {
        op: AssignOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Conditional {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
    },
    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },
    Subscript {
        base: Box<Expression>,
        index: Box<Expression>,
    },
    Member {
        base: Box<Expression>,
        field: String,
        indirect: bool,
    },
    Cast {
        target: CType,
        expression: Box<Expression>,
    },
    /// A conversion required by the C abstract machine and inserted by Sema.
    ImplicitCast {
        kind: ImplicitCastKind,
        expression: Box<Expression>,
    },
    SizeofType(CType),
    SizeofExpression(Box<Expression>),
    Alignof(CType),
    CompoundLiteral {
        ty: CType,
        initializer: Box<Initializer>,
    },
    GenericSelection {
        controlling: Box<Expression>,
        selected: Box<Expression>,
    },
    PostIncrement {
        operand: Box<Expression>,
        decrement: bool,
    },
    Comma(Vec<Expression>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImplicitCastKind {
    LValueToRValue,
    ArrayToPointerDecay,
    FunctionToPointerDecay,
    IntegralPromotion,
    FloatingConversion,
    IntegralConversion,
    PointerConversion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Plus,
    Minus,
    LogicalNot,
    BitNot,
    AddressOf,
    Dereference,
    PreIncrement,
    PreDecrement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Multiply,
    Divide,
    Remainder,
    Add,
    Subtract,
    ShiftLeft,
    ShiftRight,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
    BitAnd,
    BitXor,
    BitOr,
    LogicalAnd,
    LogicalOr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Assign,
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    ShiftLeft,
    ShiftRight,
    BitAnd,
    BitXor,
    BitOr,
}
