use crate::errors::parser::decl_error::DeclResult;
use crate::parser::ast::types::{ArrayType, ParamsType, QualType};
use crate::parser::comp_ctx::CompCtx;
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
fn resolve_decl_spec(ctx: &mut CompCtx, decl_spec: &DeclSpec) -> DeclResult<QualType> {
    let ty = ctx.type_ctx.build_type(decl_spec.ty_builder.clone());
    let qual = Qualifier::from(&decl_spec.type_quals);
    let qual_ty = QualType::new(ty, qual);
    Ok(qual_ty)
}

/// 解析 declarator, 不负责解析 decl_spec 的 storage 与 func_spec
pub fn resolve_declarator(ctx: &mut CompCtx, declarator: Declarator) -> DeclResult<DeclInfo> {
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
            DeclaratorChunkKind::Function { param } => {
                let ty = resolve_function(ctx, qual_ty, param)?;
                QualType::from(ty) // 数组本身不能接任何qual所以只能，所以提供默认的初始化？
            }
        };
    }

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
) -> DeclResult<TypeKey> {
    let size = resolve_array_size(ctx, expr)?;

    let array = ArrayType { elem_ty, size };
    // 数组类型
    let builder = TypeBuilder::Array(array);
    let ty = ctx.type_ctx.build_type(builder);
    Ok(ty)
}

/// 解析数组大小
/// todo 重构
fn resolve_array_size(ctx: &CompCtx, expr: Option<ExprKey>) -> DeclResult<ArraySize> {
    // 设置大小类型
    let expr = match expr {
        None => return Ok(ArraySize::Incomplete), // 直接回填
        Some(x) => x,
    };

    // 检查 计算
    let result = check_array_size_expr(ctx, expr)?;

    let size = match result {
        None => ArraySize::VLA(expr),    // 非常量表达式
        Some(x) => ArraySize::Static(x), // ICE
    };

    Ok(size)
}

/// 解析并计算表达式
/// # return
/// - `Err`: 检查失败，数值无效
/// - `Ok`:
///     - `Some`: 得到ICE
///     - `None`: 非编译时常量，但是通过合法性检查
fn check_array_size_expr(ctx: &CompCtx, expr_key: ExprKey) -> DeclResult<Option<usize>> {
    // 1. 检查表达式各种类型
    let expr = ctx.get_expr(expr_key);
    let ty = ctx.type_ctx.get_type(expr.ty);
    if ty.kind.is_build_in() {}

    // 2. 检查是否是常量表达式，尝试Eval
    // 2.1 尝试将常量表达式解析为usize
    // 3. 检查是否是可用的非常量表达式, VLA
    todo!("检查表达式的各种类型,")
}

/// 解析函数类型
fn resolve_function(ctx: &mut CompCtx, ret_ty: QualType, params: ParamDecl) -> DeclResult<TypeKey> {
    // 获取参数列表，可能是KR类型，这个类型理论上是不能用于声明函数类型的
    let params = resolve_params(ctx, params)?;

    // 构建类型
    let builder = TypeBuilder::new_func(params, ret_ty);

    let ty = ctx.type_ctx.build_type(builder);

    Ok(ty)
}

/// 解析参数列表
fn resolve_params(ctx: &mut CompCtx, params: ParamDecl) -> DeclResult<ParamsType> {
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
