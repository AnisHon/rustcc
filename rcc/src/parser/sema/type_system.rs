use super::super::ast::*;
use super::super::parser_core::PResult;
use super::Sema;
use crate::err::{Diagnostic, ErrorKind};
use crate::source::SourceRange;

impl Sema {
    fn error(&self, message: impl Into<String>, range: SourceRange) -> Diagnostic {
        Diagnostic::new(ErrorKind::Semantic, message, range)
    }

    pub(crate) fn validate_type(&self, ty: &CType, range: SourceRange) -> PResult<()> {
        if ty.qualifiers.is_atomic
            && matches!(ty.kind, TypeKind::Array { .. } | TypeKind::Function { .. })
        {
            return Err(self.error(
                "_Atomic cannot be applied to an array or function type",
                range,
            ));
        }
        match &ty.kind {
            TypeKind::Pointer(pointee) => self.validate_type(pointee, range),
            TypeKind::Array { element, .. } => {
                if matches!(element.kind, TypeKind::Void | TypeKind::Function { .. }) {
                    return Err(
                        self.error("array element type must be a complete object type", range)
                    );
                }
                self.validate_type(element, range)
            }
            TypeKind::Function {
                return_type,
                params,
                ..
            } => {
                if matches!(
                    return_type.kind,
                    TypeKind::Array { .. } | TypeKind::Function { .. }
                ) {
                    return Err(
                        self.error("function cannot return an array or function type", range)
                    );
                }
                self.validate_type(return_type, range)?;
                for parameter in params {
                    self.validate_type(&parameter.ty, parameter.range)?;
                }
                Ok(())
            }
            TypeKind::Complex(inner) | TypeKind::Imaginary(inner) => {
                self.validate_type(inner, range)
            }
            _ => Ok(()),
        }
    }

    pub(crate) fn require_scalar(&self, e: &Expression, where_: &str) -> PResult<()> {
        if e.ty.decay().is_scalar() {
            Ok(())
        } else {
            Err(self.error(format!("{where_} requires scalar type"), e.range))
        }
    }
    pub(crate) fn assignment_conversion(
        &self,
        target: &CType,
        expression: Expression,
    ) -> PResult<Expression> {
        let null_pointer_constant = self.const_int(&expression) == Some(0);
        let expression = self.default_conversion(expression);
        let source = expression.ty.clone();
        let allowed = self.compatible(target, &source)
            || target.is_arithmetic() && source.is_arithmetic()
            || matches!(target.kind, TypeKind::Pointer(_))
                && (matches!(source.kind, TypeKind::Pointer(_)) || null_pointer_constant);
        if !allowed {
            return Err(self.error("incompatible assignment", expression.range));
        }
        let mut result = target.clone();
        result.qualifiers = Qualifiers::default();
        Ok(self.convert_to(expression, result))
    }

    pub(crate) fn make_binary(
        &self,
        op: BinaryOp,
        left: Expression,
        right: Expression,
    ) -> PResult<Expression> {
        use BinaryOp::*;
        let span = left.range.join(right.range);
        let mut left = self.default_conversion(left);
        let mut right = self.default_conversion(right);
        let left_type = left.ty.clone();
        let right_type = right.ty.clone();
        let result = match op {
            LogicalAnd | LogicalOr => {
                if !left_type.is_scalar() || !right_type.is_scalar() {
                    return Err(self.error("logical operands must be scalar", span));
                }
                CType::int()
            }
            Less | LessEqual | Greater | GreaterEqual | Equal | NotEqual => {
                if left_type.is_arithmetic() && right_type.is_arithmetic() {
                    (left, right, _) = self.usual_arithmetic_conversions(left, right);
                } else if !(matches!(left_type.kind, TypeKind::Pointer(_))
                    && matches!(right_type.kind, TypeKind::Pointer(_)))
                {
                    return Err(self.error("invalid comparison operands", span));
                }
                CType::int()
            }
            Add | Subtract
                if matches!(left_type.kind, TypeKind::Pointer(_)) && right_type.is_integer() =>
            {
                right = self.integer_promotion(right);
                left_type
            }
            Subtract
                if matches!(left_type.kind, TypeKind::Pointer(_))
                    && matches!(right_type.kind, TypeKind::Pointer(_)) =>
            {
                CType::new(TypeKind::Long { signed: true })
            }
            Add if left_type.is_integer() && matches!(right_type.kind, TypeKind::Pointer(_)) => {
                left = self.integer_promotion(left);
                right_type
            }
            ShiftLeft | ShiftRight => {
                if !left_type.is_integer() || !right_type.is_integer() {
                    return Err(self.error("operator requires integer operands", span));
                }
                left = self.integer_promotion(left);
                right = self.integer_promotion(right);
                left.ty.clone()
            }
            Remainder | BitAnd | BitXor | BitOr => {
                if !left_type.is_integer() || !right_type.is_integer() {
                    return Err(self.error("operator requires integer operands", span));
                }
                let common;
                (left, right, common) = self.usual_arithmetic_conversions(left, right);
                common
            }
            Multiply | Divide | Add | Subtract => {
                if !left_type.is_arithmetic() || !right_type.is_arithmetic() {
                    return Err(self.error("operator requires arithmetic operands", span));
                }
                let common;
                (left, right, common) = self.usual_arithmetic_conversions(left, right);
                common
            }
        };
        Ok(Expression {
            kind: ExpressionKind::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            },
            ty: result,
            category: ValueCategory::RValue,
            range: span,
        })
    }

    pub(crate) fn default_conversion(&self, expression: Expression) -> Expression {
        let (kind, ty) = match &expression.ty.kind {
            TypeKind::Array { element, .. } => (
                ImplicitCastKind::ArrayToPointerDecay,
                CType::pointer((**element).clone()),
            ),
            TypeKind::Function { .. } => (
                ImplicitCastKind::FunctionToPointerDecay,
                CType::pointer(expression.ty.clone()),
            ),
            _ if expression.category == ValueCategory::LValue => {
                let mut ty = expression.ty.clone();
                ty.qualifiers = Qualifiers::default();
                (ImplicitCastKind::LValueToRValue, ty)
            }
            _ => return expression,
        };
        self.implicit_cast(expression, ty, kind)
    }

    pub(crate) fn integer_promotion(&self, expression: Expression) -> Expression {
        let expression = self.default_conversion(expression);
        let promoted = self.integer_promote(&expression.ty);
        if promoted == expression.ty {
            expression
        } else {
            self.implicit_cast(expression, promoted, ImplicitCastKind::IntegralPromotion)
        }
    }

    fn usual_arithmetic_conversions(
        &self,
        left: Expression,
        right: Expression,
    ) -> (Expression, Expression, CType) {
        let left = if left.ty.is_integer() {
            self.integer_promotion(left)
        } else {
            left
        };
        let right = if right.ty.is_integer() {
            self.integer_promotion(right)
        } else {
            right
        };
        let common = self.usual_arithmetic(&left.ty, &right.ty);
        let left = self.convert_to(left, common.clone());
        let right = self.convert_to(right, common.clone());
        (left, right, common)
    }

    fn convert_to(&self, expression: Expression, target: CType) -> Expression {
        if expression.ty == target {
            return expression;
        }
        let kind = if expression.ty.is_integer() && target.is_integer() {
            ImplicitCastKind::IntegralConversion
        } else if expression.ty.is_arithmetic() && target.is_arithmetic() {
            ImplicitCastKind::FloatingConversion
        } else {
            ImplicitCastKind::PointerConversion
        };
        self.implicit_cast(expression, target, kind)
    }

    fn implicit_cast(
        &self,
        expression: Expression,
        target: CType,
        kind: ImplicitCastKind,
    ) -> Expression {
        let span = expression.range;
        Expression {
            kind: ExpressionKind::ImplicitCast {
                kind,
                expression: Box::new(expression),
            },
            ty: target,
            category: ValueCategory::RValue,
            range: span,
        }
    }
    pub(crate) fn compatible(&self, a: &CType, b: &CType) -> bool {
        match (&a.kind, &b.kind) {
            (TypeKind::Void, TypeKind::Void)
            | (TypeKind::Bool, TypeKind::Bool)
            | (TypeKind::Float, TypeKind::Float)
            | (TypeKind::Double, TypeKind::Double)
            | (TypeKind::LongDouble, TypeKind::LongDouble) => true,
            (TypeKind::Char { signed: x }, TypeKind::Char { signed: y }) => x == y,
            (TypeKind::Complex(x), TypeKind::Complex(y))
            | (TypeKind::Imaginary(x), TypeKind::Imaginary(y)) => self.compatible(x, y),
            (TypeKind::Short { signed: x }, TypeKind::Short { signed: y })
            | (TypeKind::Int { signed: x }, TypeKind::Int { signed: y })
            | (TypeKind::Long { signed: x }, TypeKind::Long { signed: y })
            | (TypeKind::LongLong { signed: x }, TypeKind::LongLong { signed: y }) => x == y,
            (TypeKind::Pointer(x), TypeKind::Pointer(y)) => {
                self.compatible(x, y)
                    || matches!(x.kind, TypeKind::Void)
                    || matches!(y.kind, TypeKind::Void)
            }
            (
                TypeKind::Array {
                    element: x,
                    size: sx,
                },
                TypeKind::Array {
                    element: y,
                    size: sy,
                },
            ) => {
                self.compatible(x, y)
                    && (sx == sy
                        || matches!(sx, ArraySize::Unspecified)
                        || matches!(sy, ArraySize::Unspecified))
            }
            (TypeKind::Struct { id: x, .. }, TypeKind::Struct { id: y, .. })
            | (TypeKind::Union { id: x, .. }, TypeKind::Union { id: y, .. })
            | (TypeKind::Enum { id: x, .. }, TypeKind::Enum { id: y, .. }) => x == y,
            (
                TypeKind::Function {
                    return_type: x,
                    params: px,
                    variadic: vx,
                    has_prototype: hx,
                },
                TypeKind::Function {
                    return_type: y,
                    params: py,
                    variadic: vy,
                    has_prototype: hy,
                },
            ) => {
                self.compatible(x, y)
                    && vx == vy
                    && hx == hy
                    && px.len() == py.len()
                    && px
                        .iter()
                        .zip(py)
                        .all(|(a, b)| self.compatible(&a.ty, &b.ty))
            }
            _ => false,
        }
    }
    pub(crate) fn common_type(&self, a: &CType, b: &CType) -> Option<CType> {
        if a.is_arithmetic() && b.is_arithmetic() {
            Some(self.usual_arithmetic(a, b))
        } else if self.compatible(a, b)
            || matches!(a.kind, TypeKind::Pointer(_)) && matches!(b.kind, TypeKind::Pointer(_))
        {
            Some(a.clone())
        } else {
            None
        }
    }
    pub(crate) fn integer_promote(&self, t: &CType) -> CType {
        if matches!(
            t.kind,
            TypeKind::Bool | TypeKind::Char { .. } | TypeKind::Short { .. } | TypeKind::Enum { .. }
        ) {
            CType::int()
        } else {
            t.clone()
        }
    }
    pub(crate) fn rank(&self, t: &CType) -> (u8, bool) {
        match t.kind {
            TypeKind::Bool => (0, false),
            TypeKind::Char { signed } => (1, signed.unwrap_or(true)),
            TypeKind::Short { signed } => (2, signed),
            TypeKind::Int { signed } => (3, signed),
            TypeKind::Long { signed } => (4, signed),
            TypeKind::LongLong { signed } => (5, signed),
            _ => (3, true),
        }
    }
    pub(crate) fn usual_arithmetic(&self, a: &CType, b: &CType) -> CType {
        let complex =
            matches!(a.kind, TypeKind::Complex(_)) || matches!(b.kind, TypeKind::Complex(_));
        let imaginary =
            matches!(a.kind, TypeKind::Imaginary(_)) || matches!(b.kind, TypeKind::Imaginary(_));
        if complex || imaginary {
            let real_a = match &a.kind {
                TypeKind::Complex(x) | TypeKind::Imaginary(x) => &**x,
                _ => a,
            };
            let real_b = match &b.kind {
                TypeKind::Complex(x) | TypeKind::Imaginary(x) => &**x,
                _ => b,
            };
            let inner = self.usual_arithmetic(real_a, real_b);
            return CType::new(if complex {
                TypeKind::Complex(Box::new(inner))
            } else {
                TypeKind::Imaginary(Box::new(inner))
            });
        }
        if matches!(a.kind, TypeKind::LongDouble) || matches!(b.kind, TypeKind::LongDouble) {
            return CType::new(TypeKind::LongDouble);
        }
        if matches!(a.kind, TypeKind::Double) || matches!(b.kind, TypeKind::Double) {
            return CType::new(TypeKind::Double);
        }
        if matches!(a.kind, TypeKind::Float) || matches!(b.kind, TypeKind::Float) {
            return CType::new(TypeKind::Float);
        }
        let a = self.integer_promote(a);
        let b = self.integer_promote(b);
        let (ra, sa) = self.rank(&a);
        let (rb, sb) = self.rank(&b);
        let rank = ra.max(rb);
        let signed = if sa == sb {
            sa
        } else {
            (rb < ra || sb) && (ra < rb || sa)
        };
        CType::new(match rank {
            0..=3 => TypeKind::Int { signed },
            4 => TypeKind::Long { signed },
            _ => TypeKind::LongLong { signed },
        })
    }
}
