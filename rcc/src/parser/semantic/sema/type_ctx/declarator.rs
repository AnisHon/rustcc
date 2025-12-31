use crate::parser::ast::types::Qualifier;
use crate::parser::semantic::decl_spec::{DeclSpec, TypeQuals};
use crate::parser::semantic::declarator::{Declarator, DeclaratorChunkKind};
use crate::{
    err::parser_error::{self, ParserError, ParserResult},
    parser::{
        ast::{ExprKey, TypeKey, types::ArraySize},
        comp_ctx::CompCtx,
        semantic::{
            decl_spec::ParamDecl,
            sema::type_ctx::type_builder::{TypeBuilder, TypeBuilderKind},
        },
    },
};
use std::rc::Rc;

/// 解析 decl_spec, 不消耗decl_spec
fn resolve_decl_spec(ctx: &mut CompCtx, decl_spec: Rc<DeclSpec>) -> ParserResult<TypeKey> {
    let qualifier = Qualifier::new(&decl_spec.type_quals);
    let builder = TypeBuilder::new_with_qual(qualifier, decl_spec.kind.clone());

    let ty = ctx
        .type_ctx
        .build_type(builder)
        .map_err(|err| ParserError::from_type_error(err, decl_spec.span))?;

    Ok(ty)
}

/// 解析 declarator, 不负责解析 decl_spec 的 storage 与 func_spec
pub fn resolve_declarator(ctx: &mut CompCtx, declarator: Declarator) -> ParserResult<TypeKey> {
    use DeclaratorChunkKind::*;
    let mut ty = resolve_decl_spec(ctx, declarator.decl_spec)?;

    // 反向解析
    for chunk in declarator.chunks.into_iter().rev() {
        let builder = match chunk.kind {
            Array { expr } => resolve_array(ctx, ty, expr)?,
            Pointer { type_quals } => resolve_pointer(ty, type_quals),
            Function { param } => resolve_function(ctx, ty, param)?,
        };
        ty = ctx
            .type_ctx
            .build_type(builder)
            .map_err(|err| ParserError::from_type_error(err, chunk.span))?;
    }

    Ok(ty)
}

/// 解析数组
/// - `ctx`: 编译器上下文
/// - `elem_ty`: 当前基础类型
/// - `type_qual`:  Qualifier
/// - `expr`: 长度表达式
fn resolve_array(
    ctx: &mut CompCtx,
    elem_ty: TypeKey,
    expr: Option<ExprKey>,
) -> ParserResult<TypeBuilder> {
    // 设置大小类型
    let size = match expr {
        None => ArraySize::Incomplete,
        Some(x) => resolve_array_size(ctx, x)?,
    };

    // 数组类型
    let kind = TypeBuilderKind::Array { elem_ty, size };
    Ok(TypeBuilder::new(kind))
}

/// 解析数组大小
/// todo 重构
fn resolve_array_size(ctx: &mut CompCtx, expr: ExprKey) -> ParserResult<ArraySize> {
    let expr = ctx.pop_expr(expr);
    let expr_ty = ctx.type_ctx.get_type(expr.ty);

    // 不是 int 直接出错
    let array_size = expr.value.map(|x| x.as_intager().cloned()).flatten();
    let array_size = match array_size {
        Some(x) => x,
        None => {
            let kind = parser_error::ErrorKind::NotIntConstant;
            let error = ParserError::new(kind, expr.span);
            return Err(error);
        }
    };

    // 转换为 int constant
    let array_size = array_size.as_usize();

    Ok(ArraySize::Static(array_size))
}

/// 解析函数类型
fn resolve_function(
    ctx: &mut CompCtx,
    ret_ty: TypeKey,
    param: ParamDecl,
) -> ParserResult<TypeBuilder> {
    // 获取参数列表，可能是KR类型，这个类型理论上是不能用于声明函数类型的
    let list = match param {
        ParamDecl::Params(list) => list,
        ParamDecl::Idents(_) => {
            todo!("声明不能用K&R参数，可以默认int也可以报错，但是新标准应该是报错")
        }
    };

    // 获取参数列表类型
    let is_variadic = list.is_variadic;
    let params: Vec<_> = list
        .params
        .iter()
        .copied()
        .map(|x| ctx.get_decl(x).ty)
        .collect();

    // 构件类型
    let func = TypeBuilderKind::Function {
        ret_ty,
        params,
        is_variadic,
    };

    Ok(TypeBuilder::new(func))
}

fn resolve_pointer(elem_ty: TypeKey, quals: TypeQuals) -> TypeBuilder {
    let qualifier = Qualifier::new(&quals);
    let kind = TypeBuilderKind::Pointer { elem_ty };
    TypeBuilder::new_with_qual(qualifier, kind)
}
