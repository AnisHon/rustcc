use crate::err::parser_error::{ParserResult};
use crate::parser::ast::ExprKey;
use crate::parser::ast::exprs::{Expr, ExprKind, UnaryOpKind};
use crate::parser::comp_ctx::CompCtx;
use crate::parser::semantic::sema::expr::decay::{self, decay_expr};
use crate::parser::semantic::sema::expr::fold::fold_expr;
use crate::parser::semantic::sema::expr::ty::expr_type;
use crate::parser::semantic::sema::expr::value_type::{ValueCtx, ValueType};


/// 构建expression 折叠表达式
pub fn make_expr(ctx: &mut CompCtx, kind: ExprKind, span: Span) -> ParserResult<ExprKey> {
    // 1. 默认转换 衰变，左右值
    default_conversions(ctx, &mut kind);
    // 2. 类型推导
    let ty = expr_type(ctx, &kind, span)?;

    // 3. 尝试表达式折叠
    let value = fold_expr(ctx, &kind)?;

    let expr = Expr { kind, ty, span, value};

    Ok(expr)
}


// 衰变左右值变换
pub fn default_conversions(ctx: &mut CompCtx, kind: &mut ExprKind) {
    use ExprKind::*;
    use ValueCtx::*;
    match kind {
        DeclRef(x) => {}
        Literal(x) => {}
        ArraySubscript { base, index } => {
            *base = decay_expr(ctx, *base, NoValue);
            *index = decay_expr(ctx, *index, Value);
        }
        MemberAccess { base , ..} => {
            *base = decay_expr(ctx, *base, Value); 
        }
        SizeofType { ty } => {}
        SizeofExpr { expr } => {}
        Call { params, base } => {
            params.exprs.iter_mut().for_each(|x| {
                *x = decay_expr(ctx, *x, Value);
            });
            *base = decay_expr(ctx, *base, NoValue);
        }
        Unary { op, rhs } => {
            let value = match op.kind {
                UnaryOpKind::AddrOf => NoValue,
                _ => Value,
            };
            *rhs = decay_expr(ctx, *rhs, value);
        }
        Binary { lhs, rhs , .. } => {
            *lhs = decay_expr(ctx, *lhs, Value);
            *rhs = decay_expr(ctx, *rhs, Value);
        }
        Assign { lhs, op, rhs } => {
            *rhs = decay_expr(ctx, *rhs, Value);
        }
        Cast { expr, .. } => {}
        Ternary { cond, then_expr, else_expr, .. } => {
            *cond = decay_expr(ctx, *cond, Value);
            *then_expr = decay_expr(ctx, *then_expr, Value);
            *else_expr = decay_expr(ctx, *else_expr, Value);
        }
    };
}