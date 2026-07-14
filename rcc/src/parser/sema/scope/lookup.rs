use crate::errors::parser::scope_error::{ScopeError, ScopeResult, ScopeSource};
use crate::parser::ast::decls::decl::DeclStatus;
use crate::parser::ast::types::QualType;
use crate::parser::comp_ctx::CompCtx;
use crate::parser::sema::scope::scope_struct::{LabelSymbol, ScopeSymbol};
use crate::types::lex::token_kind::Symbol;
use crate::types::parser::ast::{DeclKey, StmtKey, TypeKey};
use crate::types::parser::common::Ident;

fn get_tag_symbol(ctx: &mut CompCtx, decl_key: DeclKey) -> Option<&mut ScopeSymbol> {
    let decl = ctx.get_decl(decl_key);
    let ty = decl.ty;
    let sym = match decl.name {
        None => return None,
        Some(x) => x.symbol,
    };
    let symbol = ctx
        .scope_mgr
        .entry_local_tag(sym)
        .or_insert_with(|| ScopeSymbol::new(sym, ty));
    Some(symbol)
}

/// 自动忽略匿名声明，不会检查类型
pub fn insert_tag_decl_unchecked(ctx: &mut CompCtx, decl_key: DeclKey) {
    let symbol = match get_tag_symbol(ctx, decl_key) {
        None => return,
        Some(x) => x,
    };
    symbol.decls.push(decl_key);
}

/// 自动忽略匿名声明，不会检查类型，必须保证不会出现重定义
pub fn insert_tag_def_unchecked(ctx: &mut CompCtx, decl_key: DeclKey) {
    let symbol = match get_tag_symbol(ctx, decl_key) {
        None => return,
        Some(x) => x,
    };
    debug_assert!(symbol.def.is_none());
    symbol.def = Some(decl_key);
}

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
    // use DeclKind::*;
    let decl = ctx.get_decl(decl_key);
    let ty = ctx.type_ctx.get_type(ty);
}

///
fn lookup_or_insert<'a>(
    ctx: &'a mut CompCtx,
    ident: &Ident,
    ty: QualType,
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
    ty: QualType,
    scope_kind: ScopeSource,
) -> ScopeResult<()> {
    let decl = ctx.get_decl(decl_key);
    debug_assert_eq!(decl.get_status(), DeclStatus::Declaration); // 必须是 不完全声明，也就是 decl
    debug_assert!(decl.name.is_some()); // 声明的 name 应该是一定存在的 
    let ident = decl.name.clone().expect("impossible");

    let symbol = lookup_or_insert(ctx, &ident, ty, scope_kind);

    // 判断是否是相同类型
    if symbol.ty != ty {
        // 不是出错
    } else {
        symbol.decls.push(decl_key);
    }

    Ok(())
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
    ty: QualType,
    scope_source: ScopeSource,
) -> Result<(), ScopeError> {
    let decl = ctx.get_decl(decl_key);
    debug_assert_eq!(decl.get_status(), DeclStatus::Definition); // 必须是 定义
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

    // // 返回所有前向声明，用于回填
    // Ok(symbol.decls.clone())
    Ok(())
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
