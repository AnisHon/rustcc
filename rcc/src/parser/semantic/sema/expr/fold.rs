
fn make_unsigned_long_long(n: u64, ty: TypeKey, span: Span) -> ExprKind {
    let kind = ExprKind::Constant(LiteralKind::Integer { suffix: Some(IntSuffix::ULL), value: n });
    todo!()
}

/// 折叠常量表达式
fn fold_expr(ctx: &mut CompCtx, expr: Expr) -> ParserResult<ExprKey> {
    let kind: ExprKind = match expr.kind {
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


fn fold_unary(ctx: &CompCtx, op: UnaryOpKind, rhs_key: ExprKey) -> ExprKind {
    let rhs = ctx.get_expr(rhs_key);
    if !rhs.kind.is_constant() {
        return 
    }

    match op {
        UnaryOpKind::Plus => ,
        UnaryOpKind::Minus =>  ,
        UnaryOpKind::Not => ,
        UnaryOpKind::BitNot => ,
        _ => rhs.kind,
    }
}

fn fold_binary(ctx: &CompCtx, lhs: ExprKey, op: BinOpKind, rhs: ExprKey) -> ExprKind {
    todo!()
}