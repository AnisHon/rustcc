use crate::err::parser_error::ParserResult;
use crate::lex::types::token::Token;
use crate::lex::types::token_kind::{Keyword, TokenKind};

use crate::parser::parser_core::Parser;
use crate::parser::types::ast::stmt::{Stmt, StmtKind};
use crate::types::span::Span;

impl Parser {

    fn is_labeled_stmt(token: &Token) -> bool {
        use Keyword::*;
        match &token.kind {
            TokenKind::Ident(_) => true,
            TokenKind::Keyword(kw) => matches!(kw, Case | Default),
            _ => false,
        }
    }

    fn is_selection_stmt(token: &Token) -> bool {
        use Keyword::*;
        match token.kind {
            TokenKind::Keyword(kw) => matches!(kw, If | Switch),
            _ => false
        }
    }

    fn is_iteration_stmt(token: &Token) -> bool {
        use Keyword::*;
        match token.kind {
            TokenKind::Keyword(kw) => matches!(kw, While | Do | For),
            _ => false
        }
    }

    fn is_jump_stmt(token: &Token) -> bool {
        use Keyword::*;
        match token.kind {
            TokenKind::Keyword(kw) => matches!(kw, Goto | Continue | Break | Return),
            _ => false
        }
    }

    pub(crate) fn parse_stmt(&mut self) -> ParserResult<Box<Stmt>> {
        use TokenKind::*;
        let lo = self.stream.span();
        let token = self.stream.peek();
        let kind = if Self::is_labeled_stmt(token) {
            self.parse_labeled_stmt()?
        } else if matches!(token.kind, LBrace) {
            self.parse_compound_stmt()?
        } else if Self::is_selection_stmt(token) {
            self.parse_selection_stmt()?
        } else if Self::is_jump_stmt(token) {
            self.parse_jump_stmt()?
        } else {
            let expr = self.parse_expr()?;
            let semi = self.expect(Semi)?.span;
            StmtKind::Expr { expr, semi }
        };
        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let stmt = Stmt::new_box(kind, span);
        Ok(stmt)
    }

    pub(crate) fn parse_labeled_stmt(&mut self) -> ParserResult<StmtKind> {
        let kind = if let Some(ident) = self.consume_ident() {
            let ident = ident.kind.into_ident().unwrap();
            let colon = self.expect(TokenKind::Colon)?.span;
            let stmt = self.parse_stmt()?;
            StmtKind::Label { ident, colon, stmt }
        } else if let Some(kw_case) = self.consume_keyword(Keyword::Case) {
            let case_span = kw_case.span;
            let expr = self.parse_expr()?;
            let colon = self.expect(TokenKind::Colon)?.span;
            let stmt = self.parse_stmt()?;
            StmtKind::Case { case_span, expr, colon, stmt }
        } else if let Some(kw_default) = self.consume_keyword(Keyword::Default) {
            let default = kw_default.span;
            let colon = self.expect(TokenKind::Colon)?.span;
            let stmt = self.parse_stmt()?;
            StmtKind::Default { default, colon, stmt }
        } else {
            unreachable!()
        };
        Ok(kind)
    }

    pub(crate) fn parse_compound_stmt(&mut self) -> ParserResult<StmtKind> {
        let l = self.expect(TokenKind::LBrace)?.span;

        let r = self.expect(TokenKind::RBrace)?.span;

        todo!()
    }

    pub(crate) fn parse_selection_stmt(&mut self) -> ParserResult<StmtKind> {
        let kind = if let Some(if_token) = self.consume_keyword(Keyword::If) {
            let if_span = if_token.span;
            let l = self.expect(TokenKind::LParen)?.span;
            let cond = self.parse_expr()?;
            let r = self.expect(TokenKind::RParen)?.span;
            let then_stmt = self.parse_stmt()?;
            let else_span;
            let else_stmt;
            if let Some(else_token) = self.consume_keyword(Keyword::Else) {
                else_span = Some(else_token.span);
                else_stmt = Some(self.parse_stmt()?);
            } else {
                else_span = None;
                else_stmt = None;
            }

            StmtKind::IfElse { if_span, l, cond, r, then_stmt, else_span, else_stmt }
        } else if let Some(switch) = self.consume_keyword(Keyword::Switch) {
            let switch_span = switch.span;
            let l = self.expect(TokenKind::LParen)?.span;
            let cond = self.parse_expr()?;
            let r = self.expect(TokenKind::RParen)?.span;
            let body = self.parse_stmt()?;

            StmtKind::Switch { switch_span, l, cond, r, body }
        } else {
            unreachable!()
        };

        Ok(kind)
    }

    pub(crate) fn parse_iteration_stmt(&mut self) -> ParserResult<StmtKind> {
        let kind = if let Some(while_token) = self.consume_keyword(Keyword::While) {
            let while_span = while_token.span;
            let l = self.expect(TokenKind::LParen)?.span;
            let cond = self.parse_expr()?;
            let r = self.expect(TokenKind::RParen)?.span;
            let body = self.parse_stmt()?;

            StmtKind::While { while_span, l, cond, r, body }
        } else if let Some(do_token) = self.consume_keyword(Keyword::Do) {
            let do_span = do_token.span;
            let body = self.parse_stmt()?;
            let while_span = self.expect_keyword(Keyword::While)?.span;
            let l = self.expect(TokenKind::LParen)?.span;
            let cond = self.parse_expr()?;
            let r = self.expect(TokenKind::RParen)?.span;
            let semi = self.expect(TokenKind::Semi)?.span;

            StmtKind::DoWhile { do_span, l, body, while_span, cond, r, semi }
        } else if if let Some(for_token) = self.consume_keyword(Keyword::For) {
            let for_span = for_token.span;
            let l = self.expect(TokenKind::LParen)?.span;

            let init = match self.check(TokenKind::Semi) {
                true => Some(self.parse_expr()?),
                false => None
            };
            let semi1 = self.expect(TokenKind::Semi)?.span;
            let cond = match self.check(TokenKind::Semi) {
                true => Some(self.parse_expr()?),
                false => None
            };
            let semi2 = self.expect(TokenKind::Semi)?.span;
            let step = match self.check(TokenKind::RParen) {
                true => Some(self.parse_expr()?),
                false => None
            };
            let r = self.expect(TokenKind::RParen)?.span;
            let body = self.parse_stmt()?;

            StmtKind::For { for_span, l, init, semi1, cond, semi2, step, r, body }
        } else {
            unreachable!()
        };
        Ok(kind)
    }

    pub(crate) fn parse_jump_stmt(&mut self) -> ParserResult<StmtKind> {
        todo!()
    }


}