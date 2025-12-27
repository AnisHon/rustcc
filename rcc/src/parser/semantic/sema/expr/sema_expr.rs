use crate::err::parser_error::{ParserResult};
use crate::lex::types::token_kind::{IntSuffix, LiteralKind};
use crate::parser::ast::{ExprKey, TypeKey};
use crate::parser::ast::exprs::{BinOpKind, Expr, ExprKind, UnaryOpKind};
use crate::parser::comp_ctx::CompCtx;
use crate::parser::semantic::sema::expr::decay;
use crate::parser::semantic::sema::expr::ty::expr_type;


/// 构建expression 折叠表达式
pub fn make_expr(ctx: &mut CompCtx, kind: ExprKind, span: Span) -> ParserResult<ExprKey> {
    // 类型衰变
    decay(ctx, &mut kind);
    // 计算类型
    let ty = expr_type(ctx, &kind, span)?;

    let expr = Expr { kind, ty, span, value: None };


    let expr = fold_expr(ctx, expr)?;

    Ok(expr)
}






