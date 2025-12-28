use crate::parser::ast::common::StructOrUnion;
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
fn is_enum(ctx: &CompCtx, decl_key: DeclKey, name: &Ident) -> ParserResult<()> {
    let decl = ctx.get_decl(decl_key);
    // 检查 decl 是否正确
    if !decl.kind.is_enum() {
        // 不同类型出错
        let error = ParserError::decl_not_match(decl_key, name.clone());
        return Err(error);
    }
    Ok(())
}

/// decl 是否是 record ，如果不是返回 DeclNotMatch 错误
fn is_record(
    ctx: &CompCtx,
    record: &StructOrUnion,
    decl_key: DeclKey,
    name: &Ident,
) -> ParserResult<()> {
    let decl = ctx.get_decl(decl_key);
    // 检查 decl 是否正确
    let res = decl
        .kind
        .as_record()
        .map(|(kind, _)| kind.kind == record.kind)
        .unwrap_or(false);

    if !res {
        // 不同类型出错
        let error = ParserError::decl_not_match(decl_key, name.clone());
        return Err(error);
    }
    Ok(())
}

/// 在当前作用域插入 enum 声明
pub fn insert_enum_ref(ctx: &mut CompCtx, name: Ident, span: Span) -> ParserResult<DeclKey> {
    // 查询同级是否已经存在声明
    let lookup_decl = ctx.scope_mgr.lookup_local_tag(name.symbol);

    // 存在，复用
    if let Some(x) = lookup_decl {
        is_enum(ctx, x, &name)?;
        return Ok(x);
    }

    // 不存在，构建
    let kind = TypeBuilderKind::new_enum(ctx);
    let builder = TypeBuilder::new(kind);
    let ty = ctx
        .type_ctx
        .build_type(builder)
        .map_err(|err| ParserError::from_type_error(err, span))?;

    // 构造 Decl
    let kind = DeclKind::Enum { enums: None };
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
    ctx.scope_mgr
        .insert_tag(name.symbol, decl_key)
        .map_err(|err| ParserError::from_scope_error(err, name.span))?;

    Ok(decl_key)
}

/// 引用 enum ，递归查找并引用
pub fn lookup_enum(ctx: &mut CompCtx, name: Ident, span: Span) -> ParserResult<DeclKey> {
    // 逐级查找
    let decl = ctx
        .scope_mgr
        .must_lookup_tag(name.symbol)
        .map_err(|err| ParserError::from_scope_error(err, span))?;

    // 检查 decl 是否正确
    is_enum(ctx, decl, &name)?;

    Ok(decl)
}

/// 构建 并将 record 插入符号表
fn build_record_and_insert(
    ctx: &mut CompCtx,
    record: StructOrUnion,
    name: Option<Ident>,
    span: Span,
) -> ParserResult<DeclKey> {
    let kind = TypeBuilderKind::new_record(ctx, record.kind);
    let builder = TypeBuilder::new(kind);
    let ty = ctx
        .type_ctx
        .build_type(builder)
        .map_err(|err| ParserError::from_type_error(err, span))?;

    // 构造 Decl
    let kind = DeclKind::Record {
        kind: record,
        fields: None,
    };
    let decl = Decl {
        storage: None,
        kind,
        name: name.clone(),
        ty,
        span,
    };

    // 存入池子
    let decl_key = ctx.insert_decl(decl);

    // 非匿名，插入符号表
    if let Some(x) = name {
        ctx.scope_mgr
            .insert_tag(x.symbol, decl_key)
            .map_err(|err| ParserError::from_scope_error(err, x.span))?;
    }

    Ok(decl_key)
}

/// 在当前作用域插入 record 声明
pub fn insert_record_ref(
    ctx: &mut CompCtx,
    record: StructOrUnion,
    name: Ident,
    span: Span,
) -> ParserResult<DeclKey> {
    // 查询同级是否已经存在声明
    let lookup_decl = ctx.scope_mgr.lookup_local_tag(name.symbol);

    // 存在，复用
    if let Some(x) = lookup_decl {
        is_record(ctx, &record, x, &name)?; // 插入声明肯定不会出现 redefined
        return Ok(x);
    }

    // 不存在，构建
    let decl = build_record_and_insert(ctx, record, Some(name), span)?;

    Ok(decl)
}

/// 在定义 record 之前，插入一个前向声明
pub fn insert_record_def(
    ctx: &mut CompCtx,
    record: StructOrUnion,
    name: Option<Ident>,
    span: Span,
) -> ParserResult<DeclKey> {
    // 查询同级是否已经存在声明
    let lookup_decl = name
        .as_ref()
        .map(|x| ctx.scope_mgr.lookup_local_tag(x.symbol))
        .flatten();

    // 存在，检查是否为非完全声明
    if let Some(x) = lookup_decl {
        let name = name.expect("lookup_decl is not null, so name can't be");
        // 临时常量表示错误类型
        enum ResErr {
            Ok,
            Redefine,
            NotMatch,
        }

        let decl = ctx.get_decl(x);

        // 检查是否是不同类型
        let res = decl
            .kind
            .as_record()
            .map(|(kind, fields)| {
                if fields.is_some() {
                    // 非 incomplete 类型重定义
                    ResErr::Redefine
                } else if kind.kind != record.kind {
                    // 非同一类型
                    ResErr::NotMatch
                } else {
                    // 无事发生
                    ResErr::Ok
                }
            })
            .unwrap_or(ResErr::NotMatch); // 如果都不是record直接

        // 检查结果
        match res {
            ResErr::NotMatch => return Err(ParserError::decl_not_match(x, name)),
            ResErr::Redefine => return Err(ParserError::redefinition(x, name)),
            ResErr::Ok => {}
        }
        // 是非完全声明可以直接使用
        return Ok(x);
    }

    // 不存在，构建
    let decl = build_record_and_insert(ctx, record, name, span)?;

    Ok(decl)
}

/// 引用 struct ，递归查找并引用
pub fn lookup_struct(
    ctx: &mut CompCtx,
    record: &StructOrUnion,
    name: Ident,
    span: Span,
) -> ParserResult<DeclKey> {
    // 逐级查找
    let decl = ctx
        .scope_mgr
        .must_lookup_tag(name.symbol)
        .map_err(|err| ParserError::from_scope_error(err, span))?;

    // 检查 decl 是否正确
    is_record(ctx, record, decl, &name)?;

    Ok(decl)
}
