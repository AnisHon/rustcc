use crate::{
    err::parser_error::ParserResult,
    parser::{
        ast::{
            exprs::ExprKey,
            types::{ArraySize, FloatSize, IntegerSize, Qualifier, Type, TypeKey, TypeKind},
        },
        comp_ctx::CompCtx,
        semantic::{
            decl_spec::{DeclSpec, ParamDecl, TypeSpec, TypeSpecKind},
            declarator::{Declarator, DeclaratorChunkKind},
        },
    },
};

/// 解析type主体部分
fn resolve_type_base(specs: &[TypeSpec]) -> ParserResult<TypeKind> {
    let mut int = None;
    let mut signed = None;
    let mut state = TypeSpecState::Init;
    let mut decl: Option<_> = None;

    for spec in specs {
        let next_state: TypeSpecState = match &spec.kind {
            TypeSpecKind::Int => {
                // 重复定义int
                if int.is_some() {
                    todo!()
                }
                int = Some(spec);
                TypeSpecState::Int
            }
            TypeSpecKind::Signed | TypeSpecKind::Unsigned => {
                // 重复定义signed unsigned
                if signed.is_some() {
                    todo!()
                }
                signed = Some(spec);
                continue;
            }
            TypeSpecKind::Void => TypeSpecState::Void,
            TypeSpecKind::Char => TypeSpecState::Char,
            TypeSpecKind::Short => TypeSpecState::Short,
            TypeSpecKind::Long => TypeSpecState::Long,
            TypeSpecKind::Float => TypeSpecState::Float,
            TypeSpecKind::Double => TypeSpecState::Double,
            TypeSpecKind::Struct(x) => {
                decl = Some(*x);
                TypeSpecState::Struct
            }
            TypeSpecKind::Union(x) => {
                decl = Some(*x);
                TypeSpecState::Union
            }
            TypeSpecKind::Enum(x) => {
                decl = Some(*x);
                TypeSpecState::Enum
            }
            TypeSpecKind::TypeName(_, x) => {
                decl = Some(*x);
                TypeSpecState::TypeName
            }
        };
        state = match combine(state, next_state) {
            Some(state) => state,
            None => {
                // 转移失败
                todo!();
            }
        };
    }

    let is_signed = signed.map(|x| x.kind.is_signed()).unwrap_or(true);

    // 查错
    match state {
        TypeSpecState::Float
        | TypeSpecState::Double
        | TypeSpecState::LongDouble
        | TypeSpecState::Struct
        | TypeSpecState::Union
        | TypeSpecState::Enum
        | TypeSpecState::TypeName => {
            // 不能组合signed unsigned
            if signed.is_some() {
                todo!()
            }

            // 不能组合int
            if int.is_some() {
                todo!()
            }
        }
        _ => {}
    }

    // 解析为TypeKind
    let kind = match state {
        TypeSpecState::Void => TypeKind::Void,
        TypeSpecState::Char => TypeKind::Integer {
            is_signed,
            size: IntegerSize::Char,
        },
        TypeSpecState::Short => TypeKind::Integer {
            is_signed,
            size: IntegerSize::Short,
        },
        TypeSpecState::Int => TypeKind::Integer {
            is_signed,
            size: IntegerSize::Int,
        },
        TypeSpecState::Long => TypeKind::Integer {
            is_signed,
            size: IntegerSize::Long,
        },
        TypeSpecState::LongLong => TypeKind::Integer {
            is_signed,
            size: IntegerSize::LongLong,
        },
        TypeSpecState::Float => TypeKind::Floating {
            size: FloatSize::Float,
        },
        TypeSpecState::Double => TypeKind::Floating {
            size: FloatSize::Double,
        },
        TypeSpecState::LongDouble => TypeKind::Floating {
            size: FloatSize::LongDouble,
        },
        TypeSpecState::Struct
        | TypeSpecState::Union
        | TypeSpecState::Enum
        | TypeSpecState::TypeName => 
            decl.unwrap().ty.kind.clone(),
        _ => todo!(), // todo 没有任何匹配
    };

    Ok(kind)
}

/// 解析decl_spec
fn resolve_decl_spec(ctx: &mut CompCtx, decl_spec: &DeclSpec) -> ParserResult<TypeKey> {
    let qualifier = decl_spec.type_quals;
    let kind = resolve_type_base(&decl_spec.type_specs)?;
    let ty = Type::new(qualifier, kind);

    // 去重
    let ty = ctx.type_ctx.get_or_set(ty);
    Ok(ty)
}

/// 解析declarator
pub fn resolve_declarator(ctx: &mut CompCtx, declarator: &Declarator) -> ParserResult<TypeKey> {
    let base_ty = resolve_decl_spec(ctx, &declarator.decl_spec)?;

    let mut ty = base_ty;

    // 解析chunks，这里一定要反着解析
    for chunk in declarator.chunks.iter().rev() {

        let new_ty = match &chunk.kind {
            // DeclaratorChunkKind::Paren { .. } => {
            //     // ignore
            //     continue;
            // }
            DeclaratorChunkKind::Array { type_qual, expr, .. } => 
                resolve_array(ctx, ty, *type_qual, expr.as_ref())?,

            DeclaratorChunkKind::Pointer { type_qual, .. } => {
                let pointer = TypeKind::Pointer { elem_ty: ty };
                Type::new(*type_qual, pointer)
            }

            DeclaratorChunkKind::Function { param, .. } => {
                let list = match param {
                    ParamDecl::Params(list) => list,
                    ParamDecl::Idents(_) => todo!(), // 声明不能用K&R参数
                };

                let is_variadic = list.is_variadic;
                let params: Vec<_> = list.params.iter().copied()
                    .map(|x| ctx.get_decl(x).ty).collect();

                let func = TypeKind::Function {
                    ret_ty: base_ty,
                    params,
                    is_variadic,
                };
                Type::new(Qualifier::default(), func)
            }
        };

        ty = ctx.type_ctx.get_or_set(new_ty);
    }

    Ok(base_ty)
}

fn resolve_array(
    ctx: &mut CompCtx, 
    base_ty: TypeKey, 
    type_qual: Qualifier, 
    expr: Option<&ExprKey>
) -> ParserResult<Type> {
    let qualifier = type_qual;
    // 设置大小类型
    let size = match expr {
        None => ArraySize::Incomplete,
        Some(x) => {
            let x = ctx.get_expr(*x);
            let expr_ty = ctx.get_type(x.ty);

            if expr_ty.kind.is_unknown() {
                todo!() // 类型未知
            }

            if !expr_ty.kind.is_integer() {
                // todo 类型不对
                todo!()
            }

            if x.is_int_constant(ctx) {
                let sz = x.get_int_constant(ctx)?;
                ArraySize::Static(sz)
            } else {
                ArraySize::VLA
            }
        }
    };
    let kind = TypeKind::Array {
        elem_ty: base_ty,
        size,
    };
    Ok(Type::new(qualifier, kind))
}

/// 状态机状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypeSpecState {
    Init,
    Void,
    Char,
    Short,
    Int,
    Long,
    LongLong,
    Float,
    Double,
    LongDouble,
    Struct,
    Union,
    Enum,
    TypeName,
}

/// 类型转换定义
fn combine(state1: TypeSpecState, state2: TypeSpecState) -> Option<TypeSpecState> {
    use TypeSpecState::*;
    match (state1, state2) {
        (Init, _) => Some(state2),
        (Void, _) => None,
        (Char, Int) => Some(Char),
        (Short, Int) => Some(Short),
        (Int, Char) => Some(Char),
        (Int, Short) => Some(Short),
        (Int, Long) => Some(Int),
        (Int, LongLong) => Some(LongLong),
        (Long, Int) => Some(Long),
        (Long, Long) => Some(LongLong),
        (Long, Double) => Some(LongDouble),
        (LongLong, Int) => Some(LongLong),
        (Float, _) => None,
        (Double, Long) => Some(LongDouble),
        (LongDouble, _) => None,
        (Struct, _) => None,
        (Union, _) => None,
        (Enum, _) => None,
        (TypeName, _) => None,
        (_, _) => None,
    }
}
