use super::super::ast::*;
use super::super::parser_core::PResult;
use super::Sema;
use crate::err::{Diagnostic, ErrorKind};
use crate::types::Span;

impl Sema {
    fn error(&self, message: impl Into<String>, span: Span) -> Diagnostic {
        Diagnostic::new(ErrorKind::Semantic, message, span)
    }

    pub(crate) fn require_scalar(&self, e: &Expression, where_: &str) -> PResult<()> {
        if e.ty.decay().is_scalar() {
            Ok(())
        } else {
            Err(self.error(format!("{where_} requires scalar type"), e.span))
        }
    }
    pub(crate) fn require_assignable(&self, to: &CType, from: &Expression) -> PResult<()> {
        let f = from.ty.decay();
        if self.compatible(to, &f)
            || to.is_arithmetic() && f.is_arithmetic()
            || matches!(to.kind, TypeKind::Pointer(_))
                && (matches!(f.kind, TypeKind::Pointer(_)) || self.const_int(from) == Some(0))
        {
            Ok(())
        } else {
            Err(self.error("incompatible assignment", from.span))
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
            (TypeKind::Struct { name: x, .. }, TypeKind::Struct { name: y, .. })
            | (TypeKind::Union { name: x, .. }, TypeKind::Union { name: y, .. })
            | (TypeKind::Enum { name: x, .. }, TypeKind::Enum { name: y, .. }) => {
                x.is_some() && x == y
            }
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
