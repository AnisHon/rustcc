//! C constant evaluation over an already-bound, typed AST.

use crate::TargetInfo;
use crate::parser::ast::{
    ArraySize, BinaryOp, CType, Expression, ExpressionKind, TypeKind, UnaryOp,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvaluationContext {
    IntegerConstantExpression,
    ArithmeticConstantExpression,
    StaticInitializer,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantValue {
    Integer(i128),
    Floating(f64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvaluationFailure {
    NotConstantExpression,
    DependsOnRuntimeValue(String),
    UndefinedBehavior(&'static str),
    DisallowedInContext(&'static str),
}

pub struct ConstantEvaluator<'a> {
    // Name lookup is injected by Sema. The evaluator remains independent of scopes and parser
    // token streams while still being able to read enum constants.
    target: &'a TargetInfo,
    lookup_integer: &'a dyn Fn(&str) -> Option<i128>,
}

impl<'a> ConstantEvaluator<'a> {
    pub fn new(target: &'a TargetInfo, lookup_integer: &'a dyn Fn(&str) -> Option<i128>) -> Self {
        Self {
            target,
            lookup_integer,
        }
    }

    pub fn evaluate(
        &self,
        expression: &Expression,
        context: EvaluationContext,
    ) -> Result<ConstantValue, EvaluationFailure> {
        // First compute a value, then enforce the stricter requirements of the use site. This
        // keeps "not evaluatable" separate from "evaluatable but disallowed here".
        let value = self.evaluate_value(expression)?;
        if context == EvaluationContext::IntegerConstantExpression
            && !matches!(value, ConstantValue::Integer(_))
        {
            return Err(EvaluationFailure::DisallowedInContext(
                "integer constant expression requires integer type",
            ));
        }
        Ok(value)
    }

    pub fn evaluate_integer(&self, expression: &Expression) -> Result<i128, EvaluationFailure> {
        match self.evaluate(expression, EvaluationContext::IntegerConstantExpression)? {
            ConstantValue::Integer(value) => Ok(value),
            ConstantValue::Floating(_) => unreachable!(),
        }
    }

    fn evaluate_value(&self, expression: &Expression) -> Result<ConstantValue, EvaluationFailure> {
        match &expression.kind {
            ExpressionKind::Integer(value) => Ok(ConstantValue::Integer(*value)),
            ExpressionKind::Floating(value) => Ok(ConstantValue::Floating(*value)),
            ExpressionKind::Character { value, .. } => Ok(ConstantValue::Integer(*value as i128)),
            ExpressionKind::Identifier { name, .. } => (self.lookup_integer)(name)
                .map(ConstantValue::Integer)
                .ok_or_else(|| EvaluationFailure::DependsOnRuntimeValue(name.clone())),
            ExpressionKind::Unary { op, operand } => self.unary(*op, operand),
            ExpressionKind::Binary { op, left, right } => self.binary(*op, left, right),
            ExpressionKind::Conditional {
                condition,
                then_expr,
                else_expr,
            } => {
                if self.integer(condition)? != 0 {
                    self.evaluate_value(then_expr)
                } else {
                    self.evaluate_value(else_expr)
                }
            }
            ExpressionKind::Cast { expression, .. }
            | ExpressionKind::ImplicitCast { expression, .. } => self.evaluate_value(expression),
            ExpressionKind::SizeofType(ty) => Ok(ConstantValue::Integer(self.layout(ty).0 as i128)),
            ExpressionKind::SizeofExpression(expression) => Ok(ConstantValue::Integer(
                self.layout(&expression.ty).0 as i128,
            )),
            ExpressionKind::Alignof(ty) => Ok(ConstantValue::Integer(self.layout(ty).1 as i128)),
            _ => Err(EvaluationFailure::NotConstantExpression),
        }
    }

    fn unary(
        &self,
        operator: UnaryOp,
        operand: &Expression,
    ) -> Result<ConstantValue, EvaluationFailure> {
        let value = self.integer(operand)?;
        Ok(ConstantValue::Integer(match operator {
            UnaryOp::Plus => value,
            UnaryOp::Minus => value
                .checked_neg()
                .ok_or(EvaluationFailure::UndefinedBehavior(
                    "signed integer overflow",
                ))?,
            UnaryOp::LogicalNot => i128::from(value == 0),
            UnaryOp::BitNot => !value,
            _ => return Err(EvaluationFailure::NotConstantExpression),
        }))
    }

    fn binary(
        &self,
        operator: BinaryOp,
        left: &Expression,
        right: &Expression,
    ) -> Result<ConstantValue, EvaluationFailure> {
        let left = self.integer(left)?;
        // Preserve C short-circuiting: the unevaluated operand must not introduce a false
        // runtime dependency or undefined-behavior diagnostic.
        if operator == BinaryOp::LogicalAnd && left == 0 {
            return Ok(ConstantValue::Integer(0));
        }
        if operator == BinaryOp::LogicalOr && left != 0 {
            return Ok(ConstantValue::Integer(1));
        }
        let right = self.integer(right)?;
        use BinaryOp::*;
        let value = match operator {
            Multiply => left.checked_mul(right),
            Divide => left.checked_div(right),
            Remainder => left.checked_rem(right),
            Add => left.checked_add(right),
            Subtract => left.checked_sub(right),
            ShiftLeft if (0..128).contains(&right) => left.checked_shl(right as u32),
            ShiftRight if (0..128).contains(&right) => left.checked_shr(right as u32),
            ShiftLeft | ShiftRight => {
                return Err(EvaluationFailure::UndefinedBehavior("invalid shift count"));
            }
            Less => Some(i128::from(left < right)),
            LessEqual => Some(i128::from(left <= right)),
            Greater => Some(i128::from(left > right)),
            GreaterEqual => Some(i128::from(left >= right)),
            Equal => Some(i128::from(left == right)),
            NotEqual => Some(i128::from(left != right)),
            BitAnd => Some(left & right),
            BitXor => Some(left ^ right),
            BitOr => Some(left | right),
            LogicalAnd => Some(i128::from(left != 0 && right != 0)),
            LogicalOr => Some(i128::from(left != 0 || right != 0)),
        }
        .ok_or(EvaluationFailure::UndefinedBehavior(
            "integer arithmetic overflow or division by zero",
        ))?;
        Ok(ConstantValue::Integer(value))
    }

    fn integer(&self, expression: &Expression) -> Result<i128, EvaluationFailure> {
        match self.evaluate_value(expression)? {
            ConstantValue::Integer(value) => Ok(value),
            ConstantValue::Floating(_) => Err(EvaluationFailure::DisallowedInContext(
                "operation requires an integer constant",
            )),
        }
    }

    fn layout(&self, ty: &CType) -> (usize, usize) {
        let bytes = |bits: u16| usize::from(bits.div_ceil(8));
        match &ty.kind {
            TypeKind::Void | TypeKind::Function { .. } => (0, 1),
            TypeKind::Bool | TypeKind::Char { .. } => {
                let size = bytes(self.target.char_width);
                (size, size)
            }
            TypeKind::Short { .. } => {
                let size = bytes(self.target.short_width);
                (size, size)
            }
            TypeKind::Int { .. } | TypeKind::Enum { .. } | TypeKind::Float => {
                let size = bytes(self.target.int_width);
                (size, size)
            }
            TypeKind::Long { .. } => {
                let size = bytes(self.target.long_width);
                (size, size)
            }
            TypeKind::LongLong { .. } | TypeKind::Double => {
                let size = bytes(self.target.long_long_width);
                (size, size)
            }
            TypeKind::Pointer(_) => (
                bytes(self.target.pointer_width),
                bytes(self.target.pointer_align),
            ),
            TypeKind::LongDouble => (16, 16),
            TypeKind::Complex(inner) | TypeKind::Imaginary(inner) => {
                let (size, alignment) = self.layout(inner);
                (size * 2, alignment)
            }
            TypeKind::Array { element, size } => {
                let (element_size, alignment) = self.layout(element);
                let count = match size {
                    ArraySize::Constant(count) => *count,
                    _ => 0,
                };
                (element_size * count, alignment)
            }
            TypeKind::Struct {
                fields: Some(fields),
                ..
            } => {
                let mut offset = 0;
                let mut alignment = 1;
                for field in fields {
                    let (size, field_alignment) = self.layout(&field.ty);
                    alignment = alignment.max(field_alignment);
                    offset = align_up(offset, field_alignment) + size;
                }
                (align_up(offset, alignment), alignment)
            }
            TypeKind::Union {
                fields: Some(fields),
                ..
            } => {
                let size = fields
                    .iter()
                    .map(|field| self.layout(&field.ty).0)
                    .max()
                    .unwrap_or(0);
                let alignment = fields
                    .iter()
                    .map(|field| self.layout(&field.ty).1)
                    .max()
                    .unwrap_or(1);
                (align_up(size, alignment), alignment)
            }
            TypeKind::Struct { fields: None, .. } | TypeKind::Union { fields: None, .. } => (0, 1),
        }
    }
}

fn align_up(value: usize, alignment: usize) -> usize {
    value.div_ceil(alignment) * alignment
}
