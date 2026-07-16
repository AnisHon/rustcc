//! Typed expression nodes.
//!
//! Every expression records its result type, value category, and source range. `ImplicitCast`
//! nodes are inserted by Sema and are intentionally visible to later constant evaluation and IR
//! lowering.

use super::{CType, DeclId, Initializer};
use crate::lex::token::StringEncoding;
use crate::source::SourceRange;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// How an expression denotes a value before context-dependent conversions.
pub enum ValueCategory {
    /// Designates an object and may be a modifiable assignment target.
    LValue,
    /// Computes a value rather than designating an object.
    RValue,
    /// Designates a function; most value contexts decay this to a pointer.
    Function,
}

#[derive(Debug, Clone, PartialEq)]
/// Semantic expression wrapper shared by every expression form.
///
/// Sema guarantees that `ty` and `category` describe the expression after the
/// conversions represented inside `kind` have been made explicit.
pub struct Expression {
    pub kind: ExpressionKind,
    pub ty: CType,
    pub category: ValueCategory,
    pub range: SourceRange,
}

#[derive(Debug, Clone, PartialEq)]
/// Payload variants for all currently represented C11 expressions.
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
/// Exact semantic conversion represented by an implicit AST node.
pub enum ImplicitCastKind {
    /// Conversion of an object designator to the stored value of that object.
    LValueToRValue,
    /// Conversion of an array expression to a pointer to its first element.
    ArrayToPointerDecay,
    /// Conversion of a function designator to a function pointer.
    FunctionToPointerDecay,
    /// Integer promotion of `_Bool`, narrow integer, bit-field, or enum operands.
    IntegralPromotion,
    /// Conversion between real floating types selected by arithmetic rules.
    FloatingConversion,
    /// Non-promotion conversion between integer types.
    IntegralConversion,
    /// Qualified, null, `void *`, or compatible object-pointer conversion.
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
