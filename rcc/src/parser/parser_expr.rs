use crate::err::parser_error;
use crate::err::parser_error::ParserResult;
use crate::lex::types::token::Token;
use crate::lex::types::token_kind::{Keyword, LiteralKind, TokenKind};
use crate::parser::ast::exprs::{ExprKey, ExprKind, Parameter};
use crate::parser::comp_ctx::CompCtx;
use crate::parser::parser_core::*;
use crate::parser::parser_decl::parse_type_name;
use crate::types::span::Span;

fn check_string(ctx: &CompCtx) -> bool {
    match &ctx.stream.peek().kind {
        TokenKind::Literal(x) => matches!(x, LiteralKind::String { .. }),
        _ => false,
    }
}

fn next_is_type_name(ctx: &CompCtx) -> bool {
    let token = ctx.stream.peek_next();
    is_type_qual(token) || is_type_spec(ctx, token)
}

fn consume_constant(ctx: &mut CompCtx) -> Option<Token> {
    let is_constant = match &ctx.stream.peek().kind {
        TokenKind::Literal(x) => !matches!(x, LiteralKind::String { .. }),
        _ => false,
    };
    next_conditional(ctx, is_constant)
}

fn consume_string(ctx: &mut CompCtx) -> Option<Token> {
    let is_string = check_string();
    next_conditional(ctx, is_string)
}

fn consume_unary_op(ctx: &mut CompCtx) -> Option<Token> {
    use TokenKind::*;
    let is_unary_op = matches!(
        ctx.stream.peek().kind,
        Amp | Star | Plus | Minus | Tilde | Bang
    );

    next_conditional(ctx, is_unary_op)
}

fn consume_assign_op(ctx: &mut CompCtx) -> Option<Token> {
    use TokenKind::*;
    let is_assign_op = matches!(
        ctx.stream.peek().kind,
        Assign
            | StarEq
            | SlashEq
            | PercentEq
            | PlusEq
            | MinusEq
            | ShlEq
            | ShrEq
            | AmpEq
            | CaretEq
            | PipeEq
    );
    next_conditional(ctx, is_assign_op)
}

fn parse_string(ctx: &mut CompCtx) -> Vec<Token> {
    let mut strings = Vec::with_capacity(1);
    while let Some(string) = consume_string(ctx) {
        strings.push(string)
    }
    strings
}

fn parse_primary_expr(ctx: &mut CompCtx) -> ParserResult<ExprKey> {
    let lo = ctx.stream.span();
    let kind = if let Some(ident) = consume_ident(ctx) {
        // ident
        ExprKind::make_decl_ref(ident)
    } else if let Some(constant) = consume_constant(ctx) {
        // constant
        ExprKind::make_literal(constant)
    } else if check_string(ctx) {
        // string
        let strings = parse_string(ctx);
        ExprKind::make_string(strings)
    } else if let Some(_) = consume(ctx, TokenKind::LParen) {
        // ( exprs )
        let expr = parse_expr(ctx)?;
        let _ = expect(ctx, TokenKind::RParen)?;
        // ExprKind::make_paren(lparen, expr, rparen)
        return Ok(expr);
    } else {
        // 匹配失败，无法恢复，报错
        // println!("error: {:?}", ctx.stream.peek());
        let kind = parser_error::ErrorKind::Expect {
            expect: "identifier, integer, float, char, string, '('".to_owned(),
        };
        let error = error_here(ctx, kind);
        return Err(error);
    };
    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);

    let expr = sema.make_expr(kind, span)?;
    Ok(expr)
}

fn parse_postfix_expr_suffix(ctx: &mut CompCtx, mut lhs: ExprKey) -> ParserResult<ExprKey> {
    use TokenKind::*;
    let lo = ctx.stream.span();
    loop {
        let kind = if let Some(lparen) = consume(ctx, LBracket) {
            // 数组访问[]
            let index = parse_expr(ctx)?;
            let rparen = expect(ctx, RBracket)?;
            ExprKind::make_index(lhs, index)
        } else if let Some(lparen) = consume(ctx, LParen) {
            // 函数调用()
            let param = parse_expr_list(ctx)?;
            let rparen = expect(ctx, RParen)?;
            ExprKind::make_call(lhs, lparen, param, rparen)
        } else if let Some(dot) = consume(ctx, Dot) {
            // 成员访问 a.b
            let ident = expect_ident(ctx)?;
            let field = ident.kind.into_ident().unwrap();
            ExprKind::make_dot(lhs, dot, field)
        } else if let Some(arrow) = consume(ctx, Arrow) {
            // 成员访问 a->b
            let ident = expect_ident(ctx)?;
            let field = ident.kind.into_ident().unwrap();
            ExprKind::make_dot(lhs, arrow, field)
        } else if let Some(op) = consumes(ctx, &[Inc, Dec]) {
            ExprKind::make_post(lhs, op)
        } else {
            break;
        };
        let hi = ctx.stream.prev_span();
        let span = Span::span(lo, hi);
        lhs = sema.make_expr(kind, span)?;
    }

    Ok(lhs)
}

fn parse_postfix_expr(ctx: &mut CompCtx) -> ParserResult<ExprKey> {
    let expr = parse_primary_expr(ctx)?;
    let expr = parse_postfix_expr_suffix(ctx, expr)?;
    Ok(expr)
}

fn parse_expr_list(ctx: &mut CompCtx) -> ParserResult<Parameter> {
    let mut param = Parameter::new();
    if check(ctx, TokenKind::RParen) {
        return Ok(param);
    }

    loop {
        let expr = parse_assign_expr(ctx)?;
        param.exprs.push(expr);
        if let Some(comma) = consume(ctx, TokenKind::Comma) {
            param.commas.push(comma.span)
        } else if check(ctx, TokenKind::RParen) {
            break;
        } else {
            let kind = parser_error::ErrorKind::Expect {
                expect: "expression".to_owned(),
            };
            return Err(error_here(ctx, kind));
        }
    }
    Ok(param)
}

fn parse_unary_expr(ctx: &mut CompCtx) -> ParserResult<ExprKey> {
    let lo = ctx.stream.span();
    let kind = if let Some(op) = consume_pair(ctx, TokenKind::Inc, TokenKind::Dec) {
        // 前置++
        let expr = parse_unary_expr(ctx)?;
        ExprKind::make_pre(op, expr)
    } else if let Some(op) = consume_unary_op(ctx) {
        // 一元运算符
        let expr = parse_cast_expr(ctx)?;
        ExprKind::make_unary(op, expr)
    } else if let Some(sizeof) = consume_keyword(ctx, Keyword::Sizeof) {
        // sizeof
        let peek_next = ctx.stream.peek_next();
        // 要求是 '(' + type_spec => typename
        if check(ctx, TokenKind::LParen) && is_type_spec(ctx, peek_next) {
            let lparen = ctx.stream.next();
            // sizeof typename
            let type_name = parse_type_name(ctx)?;
            let rparen = expect(ctx, TokenKind::RParen)?;
            ExprKind::make_size_of_type(sizeof, lparen, type_name, rparen)
        } else {
            let expr = parse_unary_expr(ctx)?;
            ExprKind::make_size_of_expr(sizeof, expr)
        }
    } else {
        // 什么都不是
        return parse_postfix_expr(ctx);
    };
    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);

    let expr = sema.make_expr(kind, span)?;
    Ok(expr)
}

fn parse_cast_expr(ctx: &mut CompCtx) -> ParserResult<ExprKey> {
    let lo = ctx.stream.span();
    let kind = if check(ctx, TokenKind::LParen) && next_is_type_name(ctx) {
        let lparen = ctx.stream.next();
        let type_name = parse_type_name(ctx)?;
        let rparen = expect(ctx, TokenKind::RParen)?;
        let expr = parse_cast_expr(ctx)?;
        ExprKind::make_cast(lparen, type_name, rparen, expr)
    } else {
        return parse_unary_expr(ctx);
    };
    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);

    let expr = sema.make_expr(kind, span)?;
    Ok(expr)
}

/// 处理multiplicative-expression的{ ("*" | "/" | "%") cast-expression }*部分
fn parse_multiplicative_expr_rhs(
    ctx: &mut CompCtx,
    lhs: ExprKey,
    lo: Span,
) -> ParserResult<ExprKey> {
    use TokenKind::*;
    if let Some(op) = consumes(ctx, &[Star, Slash, Percent]) {
        let rhs = parse_cast_expr(ctx)?;
        let hi = ctx.stream.prev_span();
        let span = Span::span(lo, hi);

        let kind = ExprKind::make_binary(lhs, op, rhs);
        let expr = sema.make_expr(kind, span)?;

        return parse_multiplicative_expr_rhs(ctx, expr, lo);
    }
    Ok(lhs)
}

/// multiplicative-expression
fn parse_multiplicative_expr(ctx: &mut CompCtx) -> ParserResult<ExprKey> {
    let lo = ctx.stream.span();
    let lhs = parse_cast_expr(ctx)?;
    let expr = parse_multiplicative_expr_rhs(ctx, lhs, lo)?;
    Ok(expr)
}

fn parse_additive_expr_rhs(ctx: &mut CompCtx, lhs: ExprKey, lo: Span) -> ParserResult<ExprKey> {
    use TokenKind::*;
    if let Some(op) = consume_pair(ctx, Plus, Minus) {
        let rhs = parse_multiplicative_expr(ctx)?;
        let hi = ctx.stream.prev_span();
        let span = Span::span(lo, hi);

        let kind = ExprKind::make_binary(lhs, op, rhs);
        let expr = sema.make_expr(kind, span)?;

        return parse_additive_expr_rhs(ctx, expr, lo);
    }
    Ok(lhs)
}

/// additive-expression
fn parse_additive_expr(ctx: &mut CompCtx) -> ParserResult<ExprKey> {
    let lo = ctx.stream.span();
    let lhs = parse_multiplicative_expr(ctx)?;
    let expr = parse_additive_expr_rhs(ctx, lhs, lo)?;
    Ok(expr)
}

fn parse_shift_expr_rhs(ctx: &mut CompCtx, lhs: ExprKey, lo: Span) -> ParserResult<ExprKey> {
    use TokenKind::*;
    if let Some(op) = consume_pair(ctx, Shl, Shr) {
        let rhs = parse_additive_expr(ctx)?;
        let hi = ctx.stream.prev_span();
        let span = Span::span(lo, hi);

        let kind = ExprKind::make_binary(lhs, op, rhs);
        let expr = sema.make_expr(kind, span)?;

        return parse_shift_expr_rhs(ctx, expr, lo);
    }
    Ok(lhs)
}

fn parse_shift_expr(ctx: &mut CompCtx) -> ParserResult<ExprKey> {
    let lo = ctx.stream.span();
    let lhs = parse_additive_expr(ctx)?;
    let expr = parse_shift_expr_rhs(ctx, lhs, lo)?;
    Ok(expr)
}

fn parse_relational_expr_rhs(ctx: &mut CompCtx, lhs: ExprKey, lo: Span) -> ParserResult<ExprKey> {
    use TokenKind::*;
    if let Some(op) = consumes(ctx, &[Lt, Gt, Le, Ge]) {
        let rhs = parse_shift_expr(ctx)?;
        let hi = ctx.stream.prev_span();
        let span = Span::span(lo, hi);

        let kind = ExprKind::make_binary(lhs, op, rhs);
        let expr = sema.make_expr(kind, span)?;

        return parse_relational_expr_rhs(ctx, expr, lo);
    }
    Ok(lhs)
}

fn parse_relational_expr(ctx: &mut CompCtx) -> ParserResult<ExprKey> {
    let lo = ctx.stream.span();
    let lhs = parse_shift_expr(ctx)?;
    let expr = parse_relational_expr_rhs(ctx, lhs, lo)?;
    Ok(expr)
}

fn parse_equality_expr_rhs(ctx: &mut CompCtx, lhs: ExprKey, lo: Span) -> ParserResult<ExprKey> {
    use TokenKind::*;
    if let Some(op) = consume_pair(ctx, Eq, Ne) {
        let rhs = parse_relational_expr(ctx)?;
        let hi = ctx.stream.prev_span();
        let span = Span::span(lo, hi);

        let kind = ExprKind::make_binary(lhs, op, rhs);
        let expr = sema.make_expr(kind, span)?;

        return parse_equality_expr_rhs(ctx, expr, lo);
    }
    Ok(lhs)
}

fn parse_equality_expr(ctx: &mut CompCtx) -> ParserResult<ExprKey> {
    let lo = ctx.stream.span();
    let expr = parse_relational_expr(ctx)?;
    let result = parse_equality_expr_rhs(ctx, expr, lo)?;
    Ok(result)
}

fn parse_and_expr_rhs(ctx: &mut CompCtx, lhs: ExprKey, lo: Span) -> ParserResult<ExprKey> {
    use TokenKind::*;
    if let Some(op) = consume(ctx, Amp) {
        let rhs = parse_equality_expr(ctx)?;
        let hi = ctx.stream.prev_span();
        let span = Span::span(lo, hi);

        let kind = ExprKind::make_binary(lhs, op, rhs);
        let expr = sema.make_expr(kind, span)?;

        return parse_and_expr_rhs(ctx, expr, lo);
    }
    Ok(lhs)
}

fn parse_and_expr(ctx: &mut CompCtx) -> ParserResult<ExprKey> {
    let lo = ctx.stream.span();
    let lhs = parse_equality_expr(ctx)?;
    let expr = parse_and_expr_rhs(ctx, lhs, lo)?;
    Ok(expr)
}

fn parse_exclusive_or_expr_rhs(ctx: &mut CompCtx, lhs: ExprKey, lo: Span) -> ParserResult<ExprKey> {
    use TokenKind::*;
    if let Some(op) = consume(ctx, Caret) {
        let rhs = parse_and_expr(ctx)?;
        let hi = ctx.stream.prev_span();
        let span = Span::span(lo, hi);

        let kind = ExprKind::make_binary(lhs, op, rhs);
        let expr = sema.make_expr(ctx, kind, span)?;

        return parse_exclusive_or_expr_rhs(ctx, expr, lo);
    }
    Ok(lhs)
}

fn parse_exclusive_or_expr(ctx: &mut CompCtx) -> ParserResult<ExprKey> {
    let lo = ctx.stream.span();
    let lhs = parse_and_expr(ctx)?;
    let expr = parse_exclusive_or_expr_rhs(ctx, lhs, lo)?;
    Ok(expr)
}

fn parse_inclusive_or_expr_rhs(ctx: &mut CompCtx, lhs: ExprKey, lo: Span) -> ParserResult<ExprKey> {
    use TokenKind::*;
    if let Some(op) = consume(ctx, Pipe) {
        let rhs = parse_exclusive_or_expr(ctx)?;
        let hi = ctx.stream.prev_span();
        let span = Span::span(lo, hi);

        let kind = ExprKind::make_binary(lhs, op, rhs);
        let expr = sema.make_expr(kind, span)?;

        return parse_inclusive_or_expr_rhs(ctx, expr, lo);
    }
    Ok(lhs)
}

fn parse_inclusive_or_expr(ctx: &mut CompCtx) -> ParserResult<ExprKey> {
    let lo = ctx.stream.span();
    let lhs = parse_exclusive_or_expr(ctx)?;
    let expr = parse_inclusive_or_expr_rhs(ctx, lhs, lo)?;
    Ok(expr)
}

fn parse_logical_and_expr_rhs(ctx: &mut CompCtx, lhs: ExprKey, lo: Span) -> ParserResult<ExprKey> {
    use TokenKind::*;
    if let Some(op) = consume(ctx, And) {
        let rhs = parse_inclusive_or_expr(ctx)?;
        let hi = ctx.stream.prev_span();
        let span = Span::span(lo, hi);

        let kind = ExprKind::make_binary(lhs, op, rhs);
        let expr = sema.make_expr(kind, span)?;

        return parse_logical_and_expr_rhs(ctx, expr, lo);
    }
    Ok(lhs)
}

fn parse_logical_and_expr(ctx: &mut CompCtx) -> ParserResult<ExprKey> {
    let lo = ctx.stream.span();
    let lhs = parse_inclusive_or_expr(ctx)?;
    let expr = parse_logical_and_expr_rhs(ctx, lhs, lo)?;
    Ok(expr)
}

fn parse_logical_or_expr_rhs(ctx: &mut CompCtx, lhs: ExprKey, lo: Span) -> ParserResult<ExprKey> {
    use TokenKind::*;
    if let Some(op) = consume(ctx, Or) {
        let rhs = parse_logical_and_expr(ctx)?;
        let hi = ctx.stream.prev_span();
        let span = Span::span(lo, hi);

        let kind = ExprKind::make_binary(lhs, op, rhs);
        let expr = sema.make_expr(kind, span)?;

        return parse_logical_or_expr_rhs(ctx, expr, lo);
    }
    Ok(lhs)
}

fn parse_logical_or_expr(ctx: &mut CompCtx) -> ParserResult<ExprKey> {
    let lo = ctx.stream.span();
    let rhs = parse_logical_and_expr(ctx)?;
    let expr = parse_logical_or_expr_rhs(ctx, rhs, lo)?;
    Ok(expr)
}

fn parse_conditional_expr(ctx: &mut CompCtx) -> ParserResult<ExprKey> {
    let lo = ctx.stream.span();
    let cond = parse_logical_or_expr(ctx)?;

    let question = consume(ctx, TokenKind::Question);

    // 不是三元表达式
    if question.is_none() {
        return Ok(cond);
    }
    // 一定是三元表达式
    let question = question.unwrap();
    let then_expr = parse_expr(ctx)?;
    let colon = expect(ctx, TokenKind::Colon)?; // 必须有 ':'
    let else_expr = parse_conditional_expr(ctx)?;
    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);

    let kind = ExprKind::make_ternary(cond, question, then_expr, colon, else_expr);
    let expr = sema.make_expr(kind, span)?;

    Ok(expr)
}

pub(crate) fn parse_assign_expr(ctx: &mut CompCtx) -> ParserResult<ExprKey> {
    let lo = ctx.stream.span();
    let lhs_key = parse_conditional_expr(ctx)?;
    let assign_op = consume_assign_op(ctx);
    if assign_op.is_none() {
        return Ok(lhs_key); // 不是赋值表达式
    }

    let lhs = ctx.get_expr(lhs_key);

    if !lhs.is_lvalue() {
        let kind = parser_error::ErrorKind::NotAssignable {
            ty: "Expression".to_owned(),
        };
        return Err(error_here(ctx, kind));
    }

    let assign_op = assign_op.unwrap();
    let rhs = parse_assign_expr(ctx)?;
    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);

    let kind = ExprKind::make_assign(lhs_key, assign_op, rhs);
    let expr = sema.make_expr(kind, span)?;

    Ok(expr)
}

fn parse_expr_rhs(ctx: &mut CompCtx, lhs: ExprKey, lo: Span) -> ParserResult<ExprKey> {
    if let Some(op) = consume(ctx, TokenKind::Comma) {
        let rhs = parse_assign_expr(ctx)?;
        let hi = ctx.stream.prev_span();
        let span = Span::span(lo, hi);

        let kind = ExprKind::make_binary(lhs, op, rhs);
        let expr = sema.make_expr(kind, span)?;

        return parse_expr_rhs(ctx, expr, lo);
    }
    Ok(lhs)
}

pub(crate) fn parse_expr(ctx: &mut CompCtx) -> ParserResult<ExprKey> {
    let lo = ctx.stream.span();
    let lhs = parse_assign_expr(ctx)?;
    let expr = parse_expr_rhs(ctx, lhs, lo)?;
    Ok(expr)
}
