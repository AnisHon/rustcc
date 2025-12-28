use crate::parser::ast::exprs::{Expr, ExprKind, UnaryOpKind};

/// 值上下文
/// - `Value`: 发生取值，意味着成为 R-Value ，发生 Decay
/// - `NoValue`: 没有发生取值，比如取址，SizeOf，所以不允许衰变，
pub enum ValueCtx {
    Value,
    NoValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueType {
    LValue,
    RValue,
    XValue, // C++保留，其实没什么用
}

impl ValueType {
    pub fn value_type(expr: &Expr) -> Self {
        use ExprKind::*;
        use UnaryOpKind::*;
        use ValueType::*;
        match &expr.kind {
            // Paren { expr, .. } => Self::value_type(expr.as_ref()),
            DeclRef(_) | ArraySubscript { .. } | MemberAccess { .. } | Assign { .. } => LValue,
            Unary { op, .. } => match op.kind {
                Deref => LValue,
                _ => RValue,
            },
            Constant(_)
            | Literal(_)
            | Call { .. }
            | SizeofExpr { .. }
            | SizeofType { .. }
            | Binary { .. }
            | Cast { .. }
            | Ternary { .. } => RValue,
        }
    }
}
