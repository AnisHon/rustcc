use crate::parser::{ast::{ExprKey, exprs::{Constant, ExprKind, UnaryOpKind}}, comp_ctx::CompCtx};


fn make_unsigned_long_long(n: u64, ty: TypeKey, span: Span) -> ExprKind {
    let kind = Constant(LiteralKind::Integer { suffix: Some(IntSuffix::ULL), value: n });
    todo!()
}

/// 尝试折叠常量表达式，可以让表达式失效（从常量池删除，使失效）
pub fn fold_expr(ctx: &mut CompCtx, kind: &ExprKind) -> ParserResult<Option<Value>> {

    let kind: ExprKind = match kind {
        // ExprKind::Paren { expr, .. } => return Ok(expr), // 折叠括号
        ExprKind::SizeofExpr { expr: sizeof_expr, .. } => {
            let sizeof_expr = ctx.get_expr(sizeof_expr);
            let ty = ctx.type_ctx.get_type_mut(sizeof_expr.ty);
            let size = ty.get_layout(ctx).size;
            make_unsigned_long_long(size, expr.ty.clone(), expr.span) // 折叠sizeof
        }
        ExprKind::SizeofType { ty, .. } => { // 折叠sizeof
            let ty = ctx.type_ctx.get_type_mut(ty);
            let size = ty.get_layout(ctx).size;
            make_unsigned_long_long(size, expr.ty.clone(), expr.span)
        }
        ExprKind::Unary { op, rhs } => // 折叠运算
            fold_unary(ctx, op.kind, rhs),
        ExprKind::Binary { lhs, op, rhs } =>  // 折叠运算
            fold_binary(ctx, lhs, op.kind, rhs),
        ExprKind::Cast { expr, .. } => return Ok(expr),  // 折叠类型转换
        ExprKind::Ternary { cond: cond_key, then_expr, else_expr} => { // 折叠三元运算
            let cond = ctx.get_expr(cond_key);
            match cond.kind.as_constant() {
                Some(x) => match x.is_true() {
                    true => return Ok(then_expr),
                    false => return Ok(else_expr),
                }
                None => ExprKind::Ternary { cond: cond_key, then_expr, else_expr },
            }
        }
        _ => return Ok(ctx.insert_expr(expr)), // 不折叠
    };

    let expr = Expr::new(kind, expr.ty, expr.span);
    let expr_key= ctx.insert_expr(expr);

    Ok(expr_key)
}


fn fold_unary(ctx: &CompCtx, op: UnaryOpKind, rhs_key: ExprKey) -> Option<Value> {
    use UnaryOpKind::*;
    let rhs = ctx.get_expr(rhs_key);
    // 不是常量表达式直接返回
    let value = rhs.value?; 

    match value {
        Constant::Float { value } {
            match op {
                Plus => value, // 完全不用变
                Minus => value. ,
                Not => ,
                BitNot => ,
                _ => return None,
            }

        }
        Constant::Intager { value }  => {}
        // 能支持一个*运算符
        Constant::String { value } => {
            match op {
                Deref => value[u8],
                _ => return None,
            }
        }
    }


}

fn fold_binary(ctx: &CompCtx, lhs: ExprKey, op: BinOpKind, rhs: ExprKey) -> Option<Value> {
    todo!()
}