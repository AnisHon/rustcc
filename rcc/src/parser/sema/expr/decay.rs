use crate::parser::comp_ctx::CompCtx;
use crate::parser::sema::expr::value_type::ValueCtx;
use crate::types::parser::ast::{types::TypeKind, ExprKey, TypeKey};

/// 处理类型衰变，也可以直接放到type里计算
pub(crate) fn decay_expr(ctx: &mut CompCtx, expr_key: ExprKey, value: ValueCtx) -> ExprKey {
    // NoValue不衰变
    match value {
        ValueCtx::NoValue => return expr_key,
        _ => {}
    }
    let expr = ctx.get_expr(expr_key);
    let ty = ctx.type_ctx.get_type(expr.ty);

    let expr = match ty.kind {
        // 数组/函数 衰变，衰变后直接成为 rvalue
        TypeKind::Array { .. } => decay_array(ctx, expr_key, expr.ty),
        TypeKind::Func { .. } => decay_function(ctx, expr_key, expr.ty),

        // 不是 数组/函数 进行左值衰变
        _ => {
            // if expr.is_lvalue() {
            //     decay_lvalue(ctx, expr_key, expr.ty)
            // } else {
            //     // 都不是，无绪衰变
            //     expr_key
            // }
            todo!()
        }
    };

    expr
}
/// 将 function 衰变为指针，只有sizeof不会衰变指针
fn decay_function(ctx: &mut CompCtx, func: ExprKey, ty: TypeKey) -> ExprKey {
    todo!()
}

// 将 array 衰变为指针
fn decay_array(ctx: &mut CompCtx, arr: ExprKey, ty: TypeKey) -> ExprKey {
    todo!()
}

fn decay_lvalue(ctx: &mut CompCtx, expr: ExprKey, ty: TypeKey) -> ExprKey {
    todo!()
}
