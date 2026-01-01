use crate::err::parser_error::ParserResult;
use crate::lex::types::token_kind::{Keyword, TokenKind};
use crate::parser::ast::stmt::StmtKey;
use crate::parser::comp_ctx::CompCtx;
use crate::parser::parser_core::*;
use crate::parser::parser_decl::parse_decl;
use crate::parser::parser_expr::parse_expr;
use crate::parser::semantic::ast::stmt::{Stmt, StmtKind};
use crate::parser::semantic::common::Ident;
use crate::types::span::Span;

fn check_labeled_stmt(ctx: &CompCtx) -> bool {
    use Keyword::*;
    let first = ctx.stream.peek();
    let second = ctx.stream.peek_next();
    match &first.kind {
        TokenKind::Ident(_) => matches!(second.kind, TokenKind::Colon),
        TokenKind::Keyword(kw) => matches!(kw, Case | Default),
        _ => false,
    }
}

fn check_selection_stmt(ctx: &CompCtx) -> bool {
    use Keyword::*;
    let token = ctx.stream.peek();
    match &token.kind {
        TokenKind::Keyword(kw) => matches!(kw, If | Switch),
        _ => false,
    }
}

fn check_iteration_stmt(ctx: &CompCtx) -> bool {
    use Keyword::*;
    let token = ctx.stream.peek();
    match &token.kind {
        TokenKind::Keyword(kw) => matches!(kw, While | Do | For),
        _ => false,
    }
}

fn check_jump_stmt(ctx: &CompCtx) -> bool {
    use Keyword::*;
    let token = ctx.stream.peek();
    match &token.kind {
        TokenKind::Keyword(kw) => matches!(kw, Goto | Continue | Break | Return),
        _ => false,
    }
}

fn check_decl(ctx: &CompCtx) -> bool {
    let token = ctx.stream.peek();
    is_type_spec(ctx, token) || is_type_qual(token) || is_storage_spec(token)
}

/// statement
/// # Arguments
/// only stmt: 只解析stmt无decl
pub(crate) fn parse_stmt(ctx: &mut CompCtx, only_stmt: bool) -> ParserResult<StmtKey> {
    let lo = ctx.stream.span();
    let kind = if check_labeled_stmt(ctx) {
        // label
        parse_labeled_stmt(ctx)?
    } else if check(ctx, TokenKind::LBrace) {
        // compound
        parse_compound_stmt(ctx, only_stmt, true)?
    } else if check_selection_stmt(ctx) {
        //
        parse_selection_stmt(ctx)?
    } else if check_jump_stmt(ctx) {
        // goto return
        parse_jump_stmt(ctx)?
    } else if check_iteration_stmt(ctx) {
        // for while
        parse_iteration_stmt(ctx)?
    } else {
        let expr = match check(ctx, TokenKind::Semi) {
            true => None,
            false => Some(parse_expr(ctx)?),
        };
        let semi = expect(ctx, TokenKind::Semi)?.span.to_pos();
        StmtKind::Expr { expr, semi }
    };
    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);

    let stmt = Stmt::new_key(ctx, kind, span);
    Ok(stmt)
}

fn parse_labeled_stmt(ctx: &mut CompCtx) -> ParserResult<StmtKind> {
    let kind = if let Some(ident) = consume_ident(ctx) {
        // label:
        let span = ident.span;
        let symbol = ident.kind.into_ident().unwrap();
        let ident = Ident { symbol, span };

        let colon = expect(ctx, TokenKind::Colon)?.span.to_pos();
        let stmt = parse_stmt(ctx, false)?;
        StmtKind::Label { ident, stmt }
    } else if let Some(kw_case) = consume_keyword(ctx, Keyword::Case) {
        // case 1 :
        let case_span = kw_case.span;
        let expr = parse_expr(ctx)?;
        let colon = expect(ctx, TokenKind::Colon)?.span.to_pos();
        let stmt = parse_stmt(ctx, false)?;
        StmtKind::Case {
            case_span,
            expr,
            colon,
            stmt,
        }
    } else if let Some(kw_default) = consume_keyword(ctx, Keyword::Default) {
        // default:
        let default = kw_default.span;
        let colon = expect(ctx, TokenKind::Colon)?.span.to_pos();
        let stmt = parse_stmt(ctx, false)?;
        StmtKind::Default {
            default,
            colon,
            stmt,
        }
    } else {
        unreachable!()
    };
    Ok(kind)
}

/// 解析 compound 语句, 负责退出decl_context
/// # Arguments
/// - `only_stmt`: 是否只应该解析statement
/// - `new_context`: 是否开上下文
pub(crate) fn parse_compound_stmt(
    ctx: &mut CompCtx,
    only_stmt: bool,
    new_context: bool,
) -> ParserResult<StmtKind> {
    // todo 符号表

    let l = expect(ctx, TokenKind::LBrace)?.span.to_pos();
    let mut stmts = Vec::new();
    loop {
        let stmt = if check(ctx, TokenKind::RBrace) {
            break;
        } else if !only_stmt && check_decl(ctx) {
            let lo = ctx.stream.span();
            let decl = parse_decl(ctx)?;
            let hi = ctx.stream.span();
            let span = Span::span(lo, hi);

            let kind = StmtKind::Decl { decl };
            Stmt::new_key(ctx, kind, span)
        } else {
            parse_stmt(ctx, false)?
        };
        stmts.push(stmt);
    }
    let r = expect(ctx, TokenKind::RBrace)?.span.to_pos();

    // 符号表退出
    let kind = StmtKind::Compound {
        l,
        stmts,
        r,
    };
    Ok(kind)
}

fn parse_selection_stmt(ctx: &mut CompCtx) -> ParserResult<StmtKind> {
    let kind = if let Some(if_token) = consume_keyword(ctx, Keyword::If) {
        // if
        let if_span = if_token.span;
        let l = expect(ctx, TokenKind::LParen)?.span.to_pos();
        let cond = parse_expr(ctx)?;
        let r = expect(ctx, TokenKind::RParen)?.span.to_pos();
        let then_stmt = parse_stmt(ctx, true)?;
        let else_span;
        let else_stmt;
        if let Some(else_token) = consume_keyword(ctx, Keyword::Else) {
            // else
            else_span = Some(else_token.span);
            else_stmt = Some(parse_stmt(ctx, true)?);
        } else {
            else_span = None;
            else_stmt = None;
        }

        StmtKind::IfElse {
            if_span,
            l,
            cond,
            r,
            then_stmt,
            else_span,
            else_stmt,
        }
    } else if let Some(switch) = consume_keyword(ctx, Keyword::Switch) {
        // switch
        let switch_span = switch.span;
        let l = expect(ctx, TokenKind::LParen)?.span.to_pos();
        let cond = parse_expr(ctx)?;
        let r = expect(ctx, TokenKind::RParen)?.span.to_pos();
        let body = parse_stmt(ctx, true)?;

        StmtKind::Switch {
            switch_span,
            l,
            expr: cond,
            r,
            body,
        }
    } else {
        unreachable!()
    };

    Ok(kind)
}

fn parse_iteration_stmt(ctx: &mut CompCtx) -> ParserResult<StmtKind> {
    let kind = if let Some(while_token) = consume_keyword(ctx, Keyword::While) {
        // while()
        let while_span = while_token.span;
        let l = expect(ctx, TokenKind::LParen)?.span.to_pos();
        let cond = parse_expr(ctx)?;
        let r = expect(ctx, TokenKind::RParen)?.span.to_pos();
        let body = parse_stmt(ctx, true)?;

        StmtKind::While {
            while_span,
            l,
            cond,
            r,
            body,
        }
    } else if let Some(do_token) = consume_keyword(ctx, Keyword::Do) {
        //do while();
        let do_span = do_token.span;
        let body = parse_stmt(ctx, true)?;
        let while_span = expect_keyword(ctx, Keyword::While)?.span;
        let l = expect(ctx, TokenKind::LParen)?.span.to_pos();
        let cond = parse_expr(ctx)?;
        let r = expect(ctx, TokenKind::RParen)?.span.to_pos();
        let semi = expect(ctx, TokenKind::Semi)?.span.to_pos();

        StmtKind::DoWhile {
            do_span,
            l,
            body,
            while_span,
            cond,
            r,
            semi,
        }
    } else if let Some(for_token) = consume_keyword(ctx, Keyword::For) {
        // for(;;)
        let for_span = for_token.span;
        let l = expect(ctx, TokenKind::LParen)?.span.to_pos();

        let init = match check(ctx, TokenKind::Semi) {
            true => Some(parse_expr(ctx)?),
            false => None,
        };
        let semi1 = expect(ctx, TokenKind::Semi)?.span.to_pos();
        let cond = match check(ctx, TokenKind::Semi) {
            true => Some(parse_expr(ctx)?),
            false => None,
        };
        let semi2 = expect(ctx, TokenKind::Semi)?.span.to_pos();
        let step = match check(ctx, TokenKind::RParen) {
            true => Some(parse_expr(ctx)?),
            false => None,
        };
        let r = expect(ctx, TokenKind::RParen)?.span.to_pos();
        let body = parse_stmt(ctx, true)?;

        StmtKind::For {
            for_span,
            l,
            init,
            semi1,
            cond,
            semi2,
            step,
            r,
            body,
        }
    } else {
        unreachable!()
    };

    Ok(kind)
}

fn parse_jump_stmt(ctx: &mut CompCtx) -> ParserResult<StmtKind> {
    let kind = if let Some(goto_token) = consume_keyword(ctx, Keyword::Goto) {
        // goto label;
        let ident = expect_ident(ctx)?;
        let span = ident.span;
        let symbol = ident.kind.into_ident().unwrap();
        let ident = Ident { span, symbol };
        let _ = expect(ctx, TokenKind::Semi)?;

        StmtKind::Goto {
            ident,
        }
    } else if let Some(continue_token) = consume_keyword(ctx, Keyword::Continue) {
        // continue;
        let continue_span = continue_token.span;
        let semi = expect(ctx, TokenKind::Semi)?.span.to_pos();
        StmtKind::Continue {
            continue_span,
            semi,
        }
    } else if let Some(break_token) = consume_keyword(ctx, Keyword::Break) {
        // break;
        let break_span = break_token.span;
        let semi = expect(ctx, TokenKind::Semi)?.span.to_pos();
        StmtKind::Break { break_span, semi }
    } else if let Some(return_token) = consume_keyword(ctx, Keyword::Return) {
        // return ;
        let return_span = return_token.span;
        let expr = match check(ctx, TokenKind::Semi) {
            true => None,
            false => Some(parse_expr(ctx)?),
        };
        let semi = expect(ctx, TokenKind::Semi)?.span.to_pos();
        StmtKind::Return {
            return_span,
            expr,
            semi,
        }
    } else {
        unreachable!()
    };

    Ok(kind)
}
