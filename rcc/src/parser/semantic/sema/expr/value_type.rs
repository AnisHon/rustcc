use crate::parser::semantic::ast::expr::Expr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueType {
    LValue,
    RValue,
    XValue, // C++保留
}

impl ValueType {

    pub fn value_type(expr: &Expr) -> Self {
        use crate::parser::semantic::ast::expr::ExprKind::*;
        use ValueType::*;
        use crate::parser::semantic::ast::expr::UnaryOpKind::*;
        match &expr.kind {
            Paren { expr, .. } => Self::value_type(expr.as_ref()),
            DeclRef(_)
            | ArraySubscript { .. }
            | MemberAccess { .. }
            | Assign { .. } => LValue,
            Unary { op, .. } => match op.kind {
                Deref => LValue,
                _ => RValue,
            }
            Constant(_)
            | Call { .. }
            | SizeofExpr { .. }
            | SizeofType { .. }
            | Binary { .. }
            | Cast { .. }
            | Ternary { .. } => RValue
        }
    }

}