use crate::err::parser_error::{self, ParserError, ParserResult};
use crate::parser::ast::types::{ArrayType, ParamsType, QualType};
use crate::parser::comp_ctx::CompCtx;
use crate::parser::sema::expr;
use crate::types::parser::ast::types::type_builder::TypeBuilder;
use crate::types::parser::ast::types::Qualifier;
use crate::types::parser::ast::{types::ArraySize, ExprKey, TypeKey};
use crate::types::parser::common::Ident;
use crate::types::parser::decl_spec::{DeclSpec, ParamDecl, StorageSpec, TypeQuals};
use crate::types::parser::declarator::{Declarator, DeclaratorChunkKind};
use crate::types::span::Span;

/// 解析 declarator 后的结果
pub struct DeclInfo {
    pub ty: QualType,
    pub name: Option<Ident>,
    pub storage: Option<StorageSpec>,
    pub span: Span,
}

/// 解析 decl_spec, 不消耗decl_spec
fn resolve_decl_spec(ctx: &mut CompCtx, decl_spec: &DeclSpec) -> ParserResult<QualType> {
    let ty = ctx.type_ctx.build_type(decl_spec.ty_builder.clone());
    let qual = Qualifier::from(&decl_spec.type_quals);
    let qual_ty = QualType::new(ty, qual);
    Ok(qual_ty)
}

/// 解析 declarator, 不负责解析 decl_spec 的 storage 与 func_spec
pub fn resolve_declarator(ctx: &mut CompCtx, declarator: Declarator) -> ParserResult<DeclInfo> {
    let decl_spec = declarator.decl_spec;

    // 先解析 declaration specifier
    let mut qual_ty = resolve_decl_spec(ctx, &decl_spec)?;

    // 反向解析 declarator
    for chunk in declarator.chunks.into_iter().rev() {
        qual_ty = match chunk.kind {
            DeclaratorChunkKind::Array { expr } => {
                let ty = resolve_array(ctx, qual_ty, expr)?;
                QualType::from(ty) // 数组本身不能接任何qual所以只能，所以提供默认的初始化？
            }
            DeclaratorChunkKind::Pointer { type_quals } => {
                resolve_pointer(ctx, qual_ty, type_quals)
            }
            DeclaratorChunkKind::Function { param } => resolve_function(ctx, qual_ty, param)?,
        };
    }

    let qual = Qualifier::from(&decl_spec.type_quals);
    let qual_ty = QualType::new(ty, qual);

    // 构建 decl_info
    let decl_info = DeclInfo {
        ty: qual_ty,
        name: declarator.name,
        storage: decl_spec.storage.clone(),
        span: declarator.span,
    };

    Ok(decl_info)
}

/// 解析数组
/// - `ctx`: 编译器上下文
/// - `elem_ty`: 当前基础类型
/// - `type_qual`:  Qualifier
/// - `expr`: 长度表达式
fn resolve_array(
    ctx: &mut CompCtx,
    elem_ty: QualType,
    expr: Option<ExprKey>,
) -> ParserResult<TypeKey> {
    // 设置大小类型
    let size = match expr {
        None => ArraySize::Incomplete,
        Some(x) => resolve_array_size(ctx, x)?,
    };

    let array = ArrayType { elem_ty, size };
    // 数组类型
    let builder = TypeBuilder::Array(array);
    let ty = ctx.type_ctx.build_type(builder);
    Ok(ty)
}

/// 解析数组大小
/// todo 重构
fn resolve_array_size(ctx: &mut CompCtx, expr: ExprKey) -> ParserResult<ArraySize> {
    expr::eval::try_as_integer(expr);

    let expr_ty = ctx.type_ctx.get_type(expr.ty);

    // 不是 int 直接出错
    let array_size = expr.value.map(|x| x.as_integer().cloned()).flatten();
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
    ret_ty: QualType,
    params: ParamDecl,
) -> ParserResult<TypeKey> {
    // 获取参数列表，可能是KR类型，这个类型理论上是不能用于声明函数类型的
    let params = resolve_params(ctx, params)?;

    // 构建类型
    let builder = TypeBuilder::new_func(params, ret_ty);

    let ty = ctx.type_ctx.build_type(builder);

    Ok(ty)
}

/// 解析参数列表
fn resolve_params(ctx: &mut CompCtx, params: ParamDecl) -> ParserResult<ParamsType> {
    // todo 做参数检查？

    let list = match params {
        ParamDecl::Params(list) => list,
        ParamDecl::Idents(_) => {
            todo!(
                "报错：声明不能用K&R参数，如果是函数定义的前向声明，应当解析全部参数后转换成Params形式"
            )
        }
    };

    // 全部转换为类型
    let params: Vec<_> = list
        .params
        .iter()
        .copied()
        .map(|x| ctx.get_decl(x).ty)
        .collect();

    let void_ty = ctx.type_ctx.new_void();
    let is_empty = params.is_empty();
    let is_first_void = params.first().map(|x| x.ty == void_ty).unwrap_or(false);

    let params = if is_first_void {
        // void 函数
        if params.len() > 1 || list.is_variadic {
            // 出错
            todo!("void must be first and only parameter")
        } else {
            // 无参
            ParamsType::Prototype {
                params: None,
                is_variadic: false,
            }
        }
    } else if is_empty {
        // 旧式空声明
        ParamsType::OldStyle
    } else {
        // 普通原型声明
        ParamsType::Prototype {
            params: Some(params),
            is_variadic: list.is_variadic,
        }
    };

    Ok(params)
}

/// 解析指针
fn resolve_pointer(ctx: &mut CompCtx, elem_ty: QualType, quals: TypeQuals) -> QualType {
    let qualifier = Qualifier::from(&quals);

    let builder = TypeBuilder::new_ptr(elem_ty);
    let ty = ctx.type_ctx.build_type(builder);
    QualType::new(ty, qualifier)
}
