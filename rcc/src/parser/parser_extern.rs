use crate::err::parser_error::ParserResult;
use crate::lex::types::token_kind::TokenKind;
use crate::parser::comp_ctx::CompCtx;
use crate::parser::parser_core::*;
use crate::parser::parser_decl::{
    parse_decl, parse_decl_after_declarator, parse_decl_prefix
};
use crate::parser::parser_stmt::parse_compound_stmt;
use crate::parser::semantic::ast::decl::DeclGroup;
use crate::parser::semantic::ast::func::{ExternalDecl, FuncDecl, FuncDef, TranslationUnit};
use crate::parser::semantic::ast::stmt::Stmt;
use crate::parser::semantic::declarator::{DeclPrefix};
use crate::types::span::Span;

fn check_decl_spec(ctx: &CompCtx) -> bool {
    let token = ctx.stream.peek();
    is_type_spec(ctx, token)
        || is_type_qual(token)
        || is_spec_qual(ctx, token)
        || is_storage_spec(token)
        || is_func_spec(ctx, token)
}

pub(crate) fn parse_translation_unit(ctx: &mut CompCtx) -> ParserResult<TranslationUnit> {
    let mut translation_unit = TranslationUnit::new();

    // 进入 File 作用域

    while !check(ctx, TokenKind::Eof) {
        parse_external_decl(ctx, &mut translation_unit)?;
    }

    // 处理暂定定义
    todo!();
    // 退出 File 作用域
    Ok(translation_unit)
}

fn parse_external_decl(
    ctx: &mut CompCtx,
    translation_unit: &mut TranslationUnit,
) -> ParserResult<()> {
    // 解析前缀
    let prefix = parse_decl_prefix(ctx)?;

    // 函数声明后 可能接 `decl_spec`(K&R) `{` 而且 declarator 一定不为空
    let is_func =
        (check_decl_spec(ctx) || check(ctx, TokenKind::LBrace)) && prefix.declarator.is_some();
    // todo 这里可能要复杂一些，检查 declaration 还是 function def, 可以搭配declarator
    let external_decl = if is_func {
        let def = parse_function_def(ctx, prefix)?;
        ExternalDecl::FunctionDefinition(def)
    } else {
        // 声明
        let group = parse_decl_after_declarator(ctx, prefix)?;
        ExternalDecl::Declaration(group)
    };
    translation_unit.push(external_decl);
    Ok(())
}

fn parse_function_def(ctx: &mut CompCtx, prefix: DeclPrefix) -> ParserResult<FuncDef> {
    debug_assert!(
        prefix.declarator.is_some(),
        "function declarator never be none"
    );
    // 进入参数作用域

    // KR函数的参数
    let decl_list = match check_decl_spec(ctx) {
        true => Some(parse_decl_list(ctx)?),
        false => None,
    };

    let hi = ctx.stream.prev_span();
    let span = Span::span(prefix.lo, hi);

    let func_decl = FuncDecl {
        declarator: prefix.declarator.expect("impossible"),
        decl_list,
        span,
    };

    // 函数声明
    let decl = sema.act_on_func_decl(func_decl)?;

    // compound stmt会调用exit_decl
    let kind = parse_compound_stmt(ctx, false, false)?;

    let hi = ctx.stream.prev_span();
    let span = Span::span(prefix.lo, hi);

    let body = Stmt::new_key(ctx, kind, span);
    let def = FuncDef { decl, body, span };

    Ok(def)
}

fn parse_decl_list(ctx: &mut CompCtx) -> ParserResult<Vec<DeclGroup>> {
    let mut list = Vec::new();
    loop {
        if check(ctx, TokenKind::LBrace) {
            break;
        }
        let group = parse_decl(ctx)?;
        list.push(group)
    }
    Ok(list)
}
