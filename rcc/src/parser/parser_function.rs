use crate::err::parser_error::ParserResult;
use crate::lex::types::token_kind::TokenKind;
use crate::parser::comp_ctx::CompCtx;
use crate::parser::parser_core::*;
use crate::parser::parser_decl::{parse_decl_after_declarator, parse_declarator};
use crate::parser::parser_stmt::parse_compound_stmt;
use crate::parser::semantic::ast::decl::DeclGroup;
use crate::parser::semantic::ast::func::{ExternalDecl, FuncDecl, FuncDef, TranslationUnit};
use crate::parser::semantic::ast::stmt::Stmt;
use crate::parser::semantic::declarator::Declarator;
use crate::parser::semantic::sema::decl::decl_context::DeclContextKind;
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
    while !check(ctx, TokenKind::Eof) {
        parse_external_decl(ctx, &mut translation_unit)?;
    }
    Ok(translation_unit)
}

fn parse_external_decl(
    ctx: &mut CompCtx,
    translation_unit: &mut TranslationUnit,
) -> ParserResult<()> {
    let lo = ctx.stream.span();
    let decl_spec = parse_decl_list(ctx)?;
    let mut declarator = Declarator::new(decl_spec);
    parse_declarator(ctx, &mut declarator)?;

    let external_decl = if check_decl_spec(ctx) || check(ctx, TokenKind::LBrace) {
        // 进入decl
        sema.enter_decl(DeclContextKind::Block);
        // KR函数的参数
        let decl_list = match check_decl_spec(ctx) {
            true => Some(parse_decl_list(ctx)?),
            false => None,
        };

        let hi = ctx.stream.prev_span();
        let span = Span::span(lo, hi);

        let func_decl = FuncDecl {
            declarator,
            decl_list,
            span,
        };

        // 函数声明
        let decl = sema.act_on_func_decl(func_decl)?;

        // compound stmt会调用exit_decl
        let kind = parse_compound_stmt(ctx, false, false)?;

        let hi = ctx.stream.prev_span();
        let span = Span::span(lo, hi);

        let body = Stmt::new_key(ctx, kind, span);
        let def = FuncDef { decl, body, span };

        ExternalDecl::FunctionDefinition(def)
    } else {
        // 声明
        let group = parse_decl_after_declarator(ctx, lo, declarator)?;
        ExternalDecl::Declaration(group)
    };
    translation_unit.push(external_decl);
    Ok(())
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
