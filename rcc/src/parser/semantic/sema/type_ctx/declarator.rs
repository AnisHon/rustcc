use crate::parser::semantic::decl_spec::TypeSpecKind;
use crate::{
    constant::typ::MAX_ARRAY_LEN,
    err::parser_error::{self, ParserError, ParserResult},
    parser::{
        ast::{
            ExprKey, TypeKey,
            exprs::ExprKind,
            types::{ArraySize, FloatSize, IntegerSize, Qualifier},
        },
        common::TypeSpecState,
        comp_ctx::CompCtx,
        semantic::{
            decl_spec::{DeclSpec, ParamDecl},
            declarator::{Declarator, DeclaratorChunkKind},
            sema::type_ctx::type_builder::{TypeBuilder, TypeBuilderKind},
        },
    },
};

/// 解析type主体部分
fn resolve_type_base(ctx: &mut CompCtx, spec: &DeclSpec) -> ParserResult<TypeBuilderKind> {
    let signed = spec.signed;

    // 查错
    match spec.base_type {
        TypeSpecState::Float
        | TypeSpecState::Double
        | TypeSpecState::LongDouble
        | TypeSpecState::Struct
        | TypeSpecState::Union
        | TypeSpecState::Enum
        | TypeSpecState::TypeName => {
            // 不能组合signed unsigned
            if signed.is_some() {
                todo!("can not combine unsigned/signed")
            }
        }
        _ => {}
    }

    let is_signed = signed.unwrap_or(false);

    // 解析为TypeKind
    let kind = match spec.base_type {
        TypeSpecState::Void => TypeBuilderKind::Void,
        TypeSpecState::Char => TypeBuilderKind::Integer {
            is_signed,
            size: IntegerSize::Char,
        },
        TypeSpecState::Short => TypeBuilderKind::Integer {
            is_signed,
            size: IntegerSize::Short,
        },
        TypeSpecState::Int => TypeBuilderKind::Integer {
            is_signed,
            size: IntegerSize::Int,
        },
        TypeSpecState::Long => TypeBuilderKind::Integer {
            is_signed,
            size: IntegerSize::Long,
        },
        TypeSpecState::LongLong => TypeBuilderKind::Integer {
            is_signed,
            size: IntegerSize::LongLong,
        },
        TypeSpecState::Float => TypeBuilderKind::Floating {
            size: FloatSize::Float,
        },
        TypeSpecState::Double => TypeBuilderKind::Floating {
            size: FloatSize::Double,
        },
        TypeSpecState::LongDouble => TypeBuilderKind::Floating {
            size: FloatSize::LongDouble,
        },
        TypeSpecState::Struct
        | TypeSpecState::Union
        | TypeSpecState::Enum
        | TypeSpecState::TypeName => {
            let base_spec = spec
                .base_spec
                .as_ref()
                .expect("when record enum typename, base_spec should not be none");
            let decl = match base_spec.kind {
                TypeSpecKind::Record(decl)
                | TypeSpecKind::Enum(decl)
                | TypeSpecKind::TypeName(_, decl) => decl,
                _ => unreachable!("expect Record Enum TypeName, got {:?}", base_spec.kind),
            };
            let decl = ctx.get_decl(decl);
            let ty = ctx.type_ctx.get_type(decl.ty);
            let kind = TypeBuilderKind::from_type_kind(ty.kind.clone());
            kind
        }
        _ => unreachable!("unknown states type: {:?}", spec.base_type),
    };

    Ok(kind)
}

/// 解析decl_spec
fn resolve_decl_spec(ctx: &mut CompCtx, decl_spec: &DeclSpec) -> ParserResult<TypeKey> {
    let type_quals = decl_spec.type_quals;
    let kind = resolve_type_base(ctx, decl_spec)?;
    let builder = TypeBuilder::new_with_qual(Qualifier::new(&type_quals), kind);

    // 去重
    let ty = ctx
        .type_ctx
        .build_type(builder)
        .map_err(|err| ParserError::from_type_error(err, decl_spec.span))?;
    Ok(ty)
}

/// 解析declarator
pub fn resolve_declarator(ctx: &mut CompCtx, declarator: &Declarator) -> ParserResult<TypeKey> {
    use DeclaratorChunkKind::*;

    let base_ty = resolve_decl_spec(ctx, &declarator.decl_spec)?;
    let mut ty = base_ty;

    // 解析chunks，这里一定要反着解析
    for chunk in declarator.chunks.iter().rev() {
        // 结合成新类型
        let new_ty = match &chunk.kind {
            // 数组类型
            Array { expr } => resolve_array(ctx, ty, *expr)?,

            // pointer 类型
            Pointer {
                type_quals: type_qual,
                ..
            } => {
                let pointer = TypeBuilderKind::Pointer { elem_ty: ty };
                TypeBuilder::new_with_qual(Qualifier::new(type_qual), pointer)
            }

            // 函数类型
            Function { param, .. } => resolve_function(ctx, ty, param)?,
            // DeclaratorChunkKind::Paren { .. } => {
            //     // ignore
            //     continue;
            // }
        };

        // 尝试去重
        ty = ctx
            .type_ctx
            .build_type(new_ty)
            .map_err(|err| ParserError::from_type_error(err, chunk.span))?;
    }

    Ok(base_ty)
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
fn resolve_array_size(ctx: &mut CompCtx, expr: ExprKey) -> ParserResult<ArraySize> {
    let expr = ctx.pop_expr(expr);
    let expr_ty = ctx.type_ctx.get_type(expr.ty);

    // 不是 int 直接出错
    if !expr_ty.is_integer() {
        let kind = parser_error::ErrorKind::NotIntConstant;
        let error = ParserError::new(kind, expr.span);
        return Err(error);
    }

    // 这里需要改成使用value
    // 如果不是 constant 就是 VLA, 否则是Static的普通静态数组
    let number = match expr.kind {
        ExprKind::Constant(x) => x,
        _ => return Ok(ArraySize::VLA),
    };

    // 获取数组大小
    let array_size = match number {
        IntConstant::Integer { value } => value,
        _ => unreachable!("not an integer"),
    };

    // 转换为 int constant
    if array_size > MAX_ARRAY_LEN {
        let error = ParserError::integer_too_large(expr.span);
        return Err(error);
    }

    Ok(ArraySize::Static(array_size as u64))
}

/// 解析函数类型
fn resolve_function(
    ctx: &mut CompCtx,
    ret_ty: TypeKey,
    param: &ParamDecl,
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
