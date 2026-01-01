use crate::{
    err::scope_error::{ScopeError, ScopeErrorKind, ScopeResult, ScopeSource},
    lex::types::token_kind::Symbol,
    parser::{
        ast::{DeclKey, StmtKey, TypeKey, decl::DeclKind},
        common::Ident,
        comp_ctx::CompCtx,
        semantic::sema::scope::scope_struct::{LabelSymbol, ScopeSymbol},
    },
};

// 检查 Type 是否一致
pub fn conflict_error_if(
    b: bool,
    ident: &Ident,
    prev: DeclKey,
    scope_source: ScopeSource,
) -> ScopeResult<()> {
    if b {
        return Ok(());
    }

    let kind = ScopeErrorKind::Conflict { prev };
    let err = ScopeError {
        kind,
        name: ident.symbol.get(),
        span: ident.span,
        scope: scope_source,
    };

    Err(err)
}

fn check_conflict(ctx: &mut CompCtx, decl_key: DeclKey, ty: TypeKey) {
    use DeclKind::*;
    let decl = ctx.get_decl(decl_key);
    let ty = ctx.type_ctx.get_type(ty);
}

///
fn lookup_or_insert<'a>(
    ctx: &'a mut CompCtx,
    ident: &Ident,
    ty: TypeKey,
    scope_kind: ScopeSource,
) -> &'a mut ScopeSymbol {
    // 选择类型
    let stack = match scope_kind {
        ScopeSource::Ident => &mut ctx.scope_mgr.idents,
        ScopeSource::Tag => &mut ctx.scope_mgr.tags,
        x => unreachable!("lookup_or_insert_decl not apply for {:?}", x),
    };

    // 拿到 ident
    let scope = stack.last_mut().expect("ident stack scope not exists");

    scope.lookup_or_insert(ident.symbol, ty)
}

/// 插入声明，不要使用这个函数插入定义，不负责回填
///
/// # Arguments
/// - `ctx`: 编译上下文
/// - `decl`: 插入声明，要求必须是声明定义，名字也不能为空
/// - `ty`: 插入声明的类型，用于检查
/// - `scope`: scope 类型 只能是 `Ident` 或者 `Tag`
///
/// # Return
/// - `Option<DeclKey>`: Definition 的 Decl
pub fn lookup_or_insert_decl(
    ctx: &mut CompCtx,
    decl_key: DeclKey,
    ty: TypeKey,
    scope_kind: ScopeSource,
) -> Option<DeclKey>  {
    let decl = ctx.get_decl(decl_key);
    debug_assert!(decl.is_decl()); // 必须是 decl
    debug_assert!(decl.name.is_some()); // 声明的 name 应该是一定存在的 
    let ident = decl.name.clone().expect("impossible");

    let symbol = lookup_or_insert(ctx, &ident, ty, scope_kind);
    symbol.decls.push(decl_key);
    symbol.def
}

/// 插入声明，不要使用这个函数插入声明，不负责回填
///
/// # Arguments
/// - `ctx`: 编译上下文
/// - `decl`: 插入声明，要求必须是定义，名字也不能为空
/// - `ty`: 插入声明的类型，用于检查
/// - `scope`: scope 类型 只能是 `Ident` 或者 `Tag`
///
/// # Return
/// - `Vec<DeclKey>`: 所有的前向声明，用于回填
pub fn lookup_or_insert_def(
    ctx: &mut CompCtx,
    decl_key: DeclKey,
    ty: TypeKey,
    scope_source: ScopeSource,
) -> Result<Vec<DeclKey>, ScopeError> {
    let decl = ctx.get_decl(decl_key);
    debug_assert!(decl.is_def()); // 必须是 decl
    debug_assert!(decl.name.is_some()); // 声明的 name 必须存在的 
    let ident = decl.name.clone().expect("impossible");

    let symbol = lookup_or_insert(ctx, &ident, ty, scope_source);

    // 检查是否重定义
    if let Some(prev) = symbol.def {
        let kind = ScopeErrorKind::Redefined { prev };
        let err = ScopeError {
            kind,
            name: ident.symbol.get(),
            scope: scope_source,
            span: ident.span,
        };
        return Err(err);
    }

    // 没有重定义
    symbol.def = Some(decl_key);

    // 返回所有前向声明，用于回填
    return Ok(symbol.decls.clone());
}

fn label_lookup_or_insert(ctx: &mut CompCtx, symbol: Symbol) -> &mut LabelSymbol {
    let scope = ctx
        .scope_mgr
        .labels
        .last_mut()
        .expect("label stack should not be none");

    scope.lookup_or_insert(symbol)
}

/// 插入 label , 不负责回填
///
/// # Arguments
/// - `stmt_key`: 必须是label statement
///
/// # Returns
/// `Vec<StmtKey>`: gotos 用于回填
pub fn lookup_or_insert_label(ctx: &mut CompCtx, stmt_key: StmtKey) -> ScopeResult<Vec<StmtKey>> {
    let stmt = ctx.get_stmt(stmt_key);
    debug_assert!(stmt.kind.is_label());
    let (ident, _) = stmt.kind.as_label().expect("impossible");
    let ident = ident.clone();

    let symbol = label_lookup_or_insert(ctx, ident.symbol);

    // 检查是否重定义
    if let Some(prev) = symbol.stmt {
        let kind = ScopeErrorKind::RedefinedLabel { prev };
        let err = ScopeError {
            kind,
            name: ident.symbol.get(),
            scope: ScopeSource::Label,
            span: ident.span,
        };
        return Err(err);
    }
    Ok(symbol.pending_gotos.clone())
}

/// 插入 goto -> label , 不负责回填
///
/// # Arguments
/// - `stmt_key`: 必须是label statement
///
/// # Argument
/// `Option<StmtKey>`: label statement
///
pub fn lookup_or_insert_goto(ctx: &mut CompCtx, stmt_key: StmtKey) -> Option<StmtKey> {
    let stmt = ctx.get_stmt(stmt_key);
    debug_assert!(stmt.kind.is_goto());
    let ident = stmt.kind.as_goto().expect("impossible").clone();

    let symbol = label_lookup_or_insert(ctx, ident.symbol);

    symbol.pending_gotos.push(stmt_key);

    symbol.stmt
}
