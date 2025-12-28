use crate::err::parser_error;
use crate::err::parser_error::{ParserError, ParserResult};
use crate::lex::types::token::Token;
use crate::lex::types::token_kind::{Keyword, TokenKind};
use crate::parser::semantic::comp_ctx::CompCtx;

/// 根据条件决定是否next
pub(crate) fn next_conditional(ctx: &mut CompCtx, cond: bool) -> Option<Token> {
    match cond {
        true => Some(ctx.stream.next()),
        false => None,
    }
}

/// 不建议用次函数检查TokenKind下的子类型
/// 对于Literal Ident行为未知
pub(crate) fn check(ctx: &CompCtx, kind: TokenKind) -> bool {
    ctx.stream.peek().kind == kind
}

pub(crate) fn checks(ctx: &CompCtx, kind: &[TokenKind]) -> bool {
    let token_kind = ctx.stream.peek().kind;
    kind.iter().any(|kind| token_kind.eq(kind))
}

pub(crate) fn check_ident(ctx: &CompCtx) -> bool {
    let kind = ctx.stream.peek().kind;
    matches!(kind, TokenKind::Ident(_))
}

pub(crate) fn check_keyword(ctx: &CompCtx, keyword: Keyword) -> bool {
    let kind = ctx.stream.peek().kind;
    kind == TokenKind::Keyword(keyword)
}

/// 同上，不建议用此函数预期TokenKind下的子类型
pub(crate) fn expects(ctx: &mut CompCtx, kinds: &[TokenKind]) -> ParserResult<Token> {
    let expected = checks(ctx, kinds);

    if expected {
        Ok(ctx.stream.next())
    } else {
        let expect: Vec<_> = kinds.iter().map(|x| x.to_string()).collect();
        let expect = expect.join(", ");
        let found = ctx.stream.peek().kind.to_string();

        let error_kind = parser_error::ErrorKind::ExpectButFound { expect, found };
        let error = error_here(ctx, error_kind);
        Err(error)
    }
}

/// 同上，不建议用此函数预期TokenKind下的子类型
pub(crate) fn expect(ctx: &mut CompCtx, kind: TokenKind) -> ParserResult<Token> {
    let expected = ctx.stream.peek().kind == kind;

    if expected {
        Ok(ctx.stream.next())
    } else {
        let expect = kind.to_string();
        let found = ctx.stream.peek().kind.to_string();

        let error_kind = parser_error::ErrorKind::ExpectButFound { expect, found };
        let error = error_here(ctx, error_kind);
        Err(error)
    }
}

pub(crate) fn expect_ident(ctx: &mut CompCtx) -> ParserResult<Token> {
    let expected = check_ident(ctx);

    if expected {
        Ok(ctx.stream.next())
    } else {
        let expect = "identifier".to_owned();
        let found: &Token = ctx.stream.peek();

        let kind = parser_error::ErrorKind::ExpectButFound {
            expect,
            found: found.kind.to_string(),
        };
        let error: ParserError = error_here(ctx, kind);
        Err(error)
    }
}

pub(crate) fn expect_keyword(ctx: &mut CompCtx, keyword: Keyword) -> ParserResult<Token> {
    let expected = check_keyword(ctx, keyword);

    if expected {
        Ok(ctx.stream.next())
    } else {
        let expect = keyword.to_string();
        let error_kind = parser_error::ErrorKind::Expect { expect };
        let error = error_here(ctx, error_kind);
        Err(error)
    }
}

pub(crate) fn expect_keyword_pair(
    ctx: &mut CompCtx,
    kw1: Keyword,
    kw2: Keyword,
) -> ParserResult<Token> {
    let kind = ctx.stream.peek().kind;
    let expected = match kind {
        TokenKind::Keyword(k) => k == kw1 || k == kw2,
        _ => false,
    };
    if expected {
        Ok(ctx.stream.next())
    } else {
        let expect = format!("{}, {}", kw1.to_string(), kw2.to_string());
        let error_kind = parser_error::ErrorKind::Expect { expect };
        let error = error_here(ctx, error_kind);
        Err(error)
    }
}

/// 同上，不建议用此函数消费TokenKind下的子类型
pub(crate) fn consumes(ctx: &mut CompCtx, kind: &[TokenKind]) -> Option<Token> {
    let is_kind = checks(ctx, kind);
    next_conditional(ctx, is_kind)
}

pub(crate) fn consume_pair(ctx: &mut CompCtx, kind1: TokenKind, kind2: TokenKind) -> Option<Token> {
    let kind = ctx.stream.peek().kind;
    let is_kind = kind == kind1 || kind == kind2;
    next_conditional(ctx, is_kind)
}

// pub(crate) fn consume_triple(ctx: &mut CompCtx, kind1: TokenKind, kind2: TokenKind, kind3: TokenKind) -> Option<Token> {
//     let kind = ctx.stream.peek().kind;
//     let is_kind = kind == kind1 || kind == kind2 || kind == kind3;
//     next_conditional(is_kind)
// }

/// 同上，不建议用此函数消费TokenKind下的子类型
pub(crate) fn consume(ctx: &mut CompCtx, kind: TokenKind) -> Option<Token> {
    let is_kind = check(ctx, kind);
    next_conditional(ctx, is_kind)
}

pub(crate) fn consume_keyword(ctx: &mut CompCtx, keyword: Keyword) -> Option<Token> {
    let is_keyword = check_keyword(ctx, keyword);
    next_conditional(ctx, is_keyword)
}

pub(crate) fn consume_keyword_pair(ctx: &mut CompCtx, kw1: Keyword, kw2: Keyword) -> Option<Token> {
    let kind = ctx.stream.peek().kind;
    let is_kw = match kind {
        TokenKind::Keyword(k) => k == kw1 || k == kw2,
        _ => false,
    };
    next_conditional(ctx, is_kw)
}

pub(crate) fn consume_ident(ctx: &mut CompCtx) -> Option<Token> {
    let is_ident = check_ident(ctx);
    next_conditional(ctx, is_ident)
}

pub(crate) fn error_here(ctx: &CompCtx, kind: parser_error::ErrorKind) -> ParserError {
    let span = ctx.stream.peek().span;
    ParserError::new(kind, span)
}

pub(crate) fn is_type_name(ctx: &CompCtx, token: &Token) -> bool {
    match token.kind {
        TokenKind::Ident(symbol) => ctx
            .scope_mgr
            .lookup_ident(symbol)
            .is_some_and(|x| ctx.get_decl(x).kind.is_type_def()),
        _ => false,
    }
}

/// (type-specifier | type-qualifier)*
pub fn is_spec_qual(ctx: &CompCtx, token: &Token) -> bool {
    is_type_spec(ctx, token) || is_type_qual(token)
}

pub fn is_type_spec(ctx: &CompCtx, token: &Token) -> bool {
    use Keyword::*;
    match token.kind {
        TokenKind::Ident(_) => is_type_name(ctx, token),
        TokenKind::Keyword(x) => matches!(
            x,
            Char | Short
                | Int
                | Long
                | Float
                | Double
                | Void
                | Signed
                | Unsigned
                | Struct
                | Union
                | Enum
        ),
        _ => false,
    }
}

pub fn is_type_qual(token: &Token) -> bool {
    use Keyword::*;
    match token.kind {
        TokenKind::Keyword(x) => matches!(x, Const | Restrict | Volatile),
        _ => false,
    }
}

pub fn is_storage_spec(token: &Token) -> bool {
    use Keyword::*;
    match token.kind {
        TokenKind::Keyword(x) => matches!(x, Typedef | Extern | Static | Auto | Register),
        _ => false,
    }
}

pub fn is_func_spec(ctx: &CompCtx, token: &Token) -> bool {
    match token.kind {
        TokenKind::Ident(_) => is_type_name(ctx, token),
        TokenKind::Keyword(x) => matches!(x, Keyword::Inline),
        _ => false,
    }
}
