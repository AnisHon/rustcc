use crate::err::scope_error::ScopeSource;
use crate::parser::ast::TypeKey;
use crate::parser::ast::common::{RecordKind, StructOrUnion};
use crate::parser::semantic::sema::scope::lookup::{
    conflict_error_if, lookup_or_insert_decl, lookup_or_insert_def,
};
use crate::{
    err::parser_error::{ParserError, ParserResult},
    parser::{
        ast::{
            DeclKey,
            decl::{Decl, DeclKind},
        },
        common::Ident,
        comp_ctx::CompCtx,
        semantic::sema::type_ctx::type_builder::{TypeBuilder, TypeBuilderKind},
    },
    types::span::Span,
};

/// decl 是否是 enum ，如果不是返回 DeclNotMatch 错误
fn is_enum(ctx: &CompCtx, ty: TypeKey) -> bool {
    let ty = ctx.type_ctx.get_type(ty);
    ty.kind.is_enum()
}

/// decl 是否是 record ，如果不是返回 DeclNotMatch 错误
fn is_record(ctx: &CompCtx, kind: RecordKind, ty: TypeKey) -> bool {
    let ty = ctx.type_ctx.get_type(ty);
    ty.kind
        .as_record()
        .map(|(kind1, _, _)| *kind1 == kind)
        .unwrap_or(false)
}

/// 在当前作用域插入 enum 声明
pub fn insert_enum_decl(ctx: &mut CompCtx, name: Ident, span: Span) -> ParserResult<DeclKey> {
    // 查询同级是否已经存在声明
    let symbol = ctx.scope_mgr.lookup_local_tag(name.symbol);
    let ty = match symbol {
        Some(x) => {
            // 检查是否为同一个
            conflict_error_if(is_enum(ctx, x.ty), &name, x.get_decl(), ScopeSource::Tag)?;
            x.ty
        }
        None => {
            // 不存在，构建类型
            let kind = TypeBuilderKind::new_enum(ctx);
            let builder = TypeBuilder::new(kind);
            ctx.type_ctx
                .build_type(builder)
                .map_err(|err| ParserError::from_type_error(err, span))?
        }
    };

    // 构造 DeclDecl
    let kind = DeclKind::EnumDef { enums: None };
    let decl = Decl {
        storage: None,
        kind,
        name: Some(name.clone()),
        ty,
        span,
    };

    // 存入池子
    let decl_key = ctx.insert_decl(decl);

    // 插入符号表
    let enum_def = lookup_or_insert_decl(ctx, decl_key, ty, ScopeSource::Tag);

    // 设置 definition
    let decl = ctx.get_decl_mut(decl_key);
    match &mut decl.kind {
        DeclKind::EnumDecl { def } => *def = enum_def,
        _ => unreachable!(),
    }

    Ok(decl_key)
}

/// 在当前作用域插入 record 声明
pub fn insert_record_decl(
    ctx: &mut CompCtx,
    record: StructOrUnion,
    name: Ident,
    span: Span,
) -> ParserResult<DeclKey> {
    // 查询同级是否已经存在声明
    let symbol = ctx.scope_mgr.lookup_local_tag(name.symbol);
    let ty = match symbol {
        Some(x) => {
            // 检查tag声明是否相同
            conflict_error_if(
                is_record(ctx, record.kind, x.ty),
                &name,
                x.get_decl(),
                ScopeSource::Tag,
            )?;
            x.ty
        }
        None => {
            // 不存在，构建类型
            let kind = TypeBuilderKind::new_enum(ctx);
            let builder = TypeBuilder::new(kind);
            ctx.type_ctx
                .build_type(builder)
                .map_err(|err| ParserError::from_type_error(err, span))?
        }
    };

    // 构造 DeclDecl
    let kind = DeclKind::RecordDecl {
        kind: record,
        def: None,
    };
    let decl = Decl {
        storage: None,
        kind,
        name: Some(name.clone()),
        ty,
        span,
    };

    // 存入池子
    let decl_key = ctx.insert_decl(decl);

    // 插入符号表
    let record_def = lookup_or_insert_decl(ctx, decl_key, ty, ScopeSource::Tag);

    // 设置 definition
    let decl = ctx.get_decl_mut(decl_key);
    match &mut decl.kind {
        DeclKind::EnumDecl { def } => *def = record_def,
        _ => unreachable!(),
    }

    Ok(decl_key)
}

/// 填充 record 的前向声明
pub fn fill_record_fwd_ref(ctx: &mut CompCtx, definition: DeclKey, decls: Vec<DeclKey>) {
    for decl in decls.into_iter() {
        let decl = ctx.get_decl_mut(decl);
        assert!(decl.kind.is_record_decl());
        match &mut decl.kind {
            DeclKind::RecordDecl { def, .. } => {
                *def = Some(definition);
            }
            _ => unreachable!(),
        }
    }
}

/// 插入 record 定义
pub fn insert_record_def(
    ctx: &mut CompCtx,
    kind: DeclKind,
    name: Ident,
    span: Span,
) -> ParserResult<DeclKey> {
    assert!(kind.is_record_def());
    let record_kind = match &kind {
        DeclKind::RecordDef { kind, .. } => kind.kind.clone(),
        _ => unreachable!(),
    };

    let symbol = ctx.scope_mgr.lookup_local_tag(name.symbol);
    let ty = match symbol {
        Some(x) => {
            // 检查tag声明是否相同
            conflict_error_if(
                is_record(ctx, record_kind, x.ty),
                &name,
                x.get_decl(),
                ScopeSource::Tag,
            )?;
            x.ty
        }
        None => {
            // 不存在，构建类型
            let kind = TypeBuilderKind::new_enum(ctx);
            let builder = TypeBuilder::new(kind);
            ctx.type_ctx
                .build_type(builder)
                .map_err(|err| ParserError::from_type_error(err, span))?
        }
    };

    // 构建 decl
    let decl = Decl {
        storage: None,
        kind,
        name: Some(name.clone()),
        ty,
        span,
    };

    let def = ctx.insert_decl(decl);

    // 添加到符号表
    let decls = lookup_or_insert_def(ctx, def, ty, ScopeSource::Tag)?;
    // 填充前向引用
    fill_record_fwd_ref(ctx, def, decls);

    Ok(def)
}
