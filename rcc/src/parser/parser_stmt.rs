use crate::err::parser_error::ParserResult;
use crate::lex::types::token_kind::{Keyword, TokenKind};
use crate::parser::parser_core::Parser;
use crate::parser::types::ast::stmt::{Stmt, StmtKind};
use crate::parser::types::common::Ident;
use crate::parser::types::sema::decl::decl_context::DeclContextKind;
use crate::types::span::Span;

impl Parser {

    fn check_labeled_stmt(&self) -> bool {
        use Keyword::*;
        let first = self.stream.peek();
        let second = self.stream.peek_next();
        match &first.kind {
            TokenKind::Ident(_) => matches!(second.kind, TokenKind::Colon),
            TokenKind::Keyword(kw) => matches!(kw, Case | Default),
            _ => false,
        }
    }

    fn check_selection_stmt(&self) -> bool {
        use Keyword::*;
        let token = self.stream.peek();
        match &token.kind {
            TokenKind::Keyword(kw) => matches!(kw, If | Switch),
            _ => false
        }
    }

    fn check_iteration_stmt(&self) -> bool {
        use Keyword::*;
        let token = self.stream.peek();
        match &token.kind {
            TokenKind::Keyword(kw) => matches!(kw, While | Do | For),
            _ => false
        }
    }

    fn check_jump_stmt(&self) -> bool {
        use Keyword::*;
        let token = self.stream.peek();
        match &token.kind {
            TokenKind::Keyword(kw) => matches!(kw, Goto | Continue | Break | Return),
            _ => false
        }
    }

    fn check_decl(&self) -> bool {
        let token = self.stream.peek();
        self.is_type_spec(token) || self.is_type_qual(token) || self.is_storage_spec(token)
    }

    /// statement
    /// # Arguments
    /// only stmt: 只解析stmt无decl
    pub(crate) fn parse_stmt(&mut self, only_stmt: bool) -> ParserResult<Box<Stmt>> {
        let lo = self.stream.span();
        let kind = if self.check_labeled_stmt() {
            self.parse_labeled_stmt()?
        } else if self.check(TokenKind::LBrace) {
            self.parse_compound_stmt(only_stmt, true)?
        } else if self.check_selection_stmt() {
            self.parse_selection_stmt()?
        } else if self.check_jump_stmt() {
            self.parse_jump_stmt()?
        } else if self.check_iteration_stmt() {
            self.parse_iteration_stmt()?
        } else {
            let expr = match self.check(TokenKind::Semi) {
                true => None,
                false => Some(self.parse_expr()?)
            };
            let semi = self.expect(TokenKind::Semi)?.span.to_pos();
            StmtKind::Expr { expr, semi }
        };
        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let stmt = Stmt::new_box(kind, span);
        Ok(stmt)
    }

    fn parse_labeled_stmt(&mut self) -> ParserResult<StmtKind> {
        let kind = if let Some(ident) = self.consume_ident() {
            // label:
            let span = ident.span;
            let symbol = ident.kind.into_ident().unwrap();
            let ident = Ident{ symbol, span };

            let colon = self.expect(TokenKind::Colon)?.span.to_pos();
            let stmt = self.parse_stmt(false)?;
            StmtKind::Label { ident, colon, stmt }
        } else if let Some(kw_case) = self.consume_keyword(Keyword::Case) {
            // case 1 :
            let case_span = kw_case.span;
            let expr = self.parse_expr()?;
            let colon = self.expect(TokenKind::Colon)?.span.to_pos();
            let stmt = self.parse_stmt(false)?;
            StmtKind::Case { case_span, expr, colon, stmt }
        } else if let Some(kw_default) = self.consume_keyword(Keyword::Default) {
            // default:
            let default = kw_default.span;
            let colon = self.expect(TokenKind::Colon)?.span.to_pos();
            let stmt = self.parse_stmt(false)?;
            StmtKind::Default { default, colon, stmt }
        } else {
            unreachable!()
        };
        Ok(kind)
    }

    /// 解析 compound 语句, 负责退出decl_context
    /// # Arguments
    /// - `only_stmt`: 是否只应该解析statement
    /// - `new_context`: 是否开上下文
    pub(crate) fn parse_compound_stmt(&mut self, only_stmt: bool, new_context: bool) -> ParserResult<StmtKind> {
        if new_context {
            self.sema.enter_decl(DeclContextKind::Block);
        }
        
        let l = self.expect(TokenKind::LBrace)?.span.to_pos();
        let mut stmts = Vec::new();
        loop {

            let stmt = if self.check(TokenKind::RBrace) {
                break
            } else if !only_stmt && self.check_decl() {
                let lo = self.stream.span();
                let decl = self.parse_decl()?;
                let hi = self.stream.span();
                let span = Span::span(lo, hi);
                
                let kind = StmtKind::Decl { decl };
                Stmt::new_box(kind, span)
            } else {
                self.parse_stmt(false)?
            };
            stmts.push(stmt);
        }
        let r = self.expect(TokenKind::RBrace)?.span.to_pos();

        let context = self.sema.exit_decl();
        
        let kind = StmtKind::Compound { l, stmts, r, context };
        Ok(kind)
    }

    fn parse_selection_stmt(&mut self) -> ParserResult<StmtKind> {
        let kind = if let Some(if_token) = self.consume_keyword(Keyword::If) {
            // if
            let if_span = if_token.span;
            let l = self.expect(TokenKind::LParen)?.span.to_pos();
            let cond = self.parse_expr()?;
            let r = self.expect(TokenKind::RParen)?.span.to_pos();
            let then_stmt = self.parse_stmt(true)?;
            let else_span;
            let else_stmt;
            if let Some(else_token) = self.consume_keyword(Keyword::Else) {
                // else
                else_span = Some(else_token.span);
                else_stmt = Some(self.parse_stmt(true)?);
            } else {
                else_span = None;
                else_stmt = None;
            }

            StmtKind::IfElse { if_span, l, cond, r, then_stmt, else_span, else_stmt }
        } else if let Some(switch) = self.consume_keyword(Keyword::Switch) {
            // switch
            let switch_span = switch.span;
            let l = self.expect(TokenKind::LParen)?.span.to_pos();
            let cond = self.parse_expr()?;
            let r = self.expect(TokenKind::RParen)?.span.to_pos();
            let body = self.parse_stmt(true)?;

            StmtKind::Switch { switch_span, l, expr: cond, r, body }
        } else {
            unreachable!()
        };

        Ok(kind)
    }

    fn parse_iteration_stmt(&mut self) -> ParserResult<StmtKind> {
        let kind = if let Some(while_token) = self.consume_keyword(Keyword::While) {
            // while()
            let while_span = while_token.span;
            let l = self.expect(TokenKind::LParen)?.span.to_pos();
            let cond = self.parse_expr()?;
            let r = self.expect(TokenKind::RParen)?.span.to_pos();
            let body = self.parse_stmt(true)?;

            StmtKind::While { while_span, l, cond, r, body }
        } else if let Some(do_token) = self.consume_keyword(Keyword::Do) {
            //do while();
            let do_span = do_token.span;
            let body = self.parse_stmt(true)?;
            let while_span = self.expect_keyword(Keyword::While)?.span;
            let l = self.expect(TokenKind::LParen)?.span.to_pos();
            let cond = self.parse_expr()?;
            let r = self.expect(TokenKind::RParen)?.span.to_pos();
            let semi = self.expect(TokenKind::Semi)?.span.to_pos();

            StmtKind::DoWhile { do_span, l, body, while_span, cond, r, semi }
        } else if let Some(for_token) = self.consume_keyword(Keyword::For) {
            // for(;;)
            let for_span = for_token.span;
            let l = self.expect(TokenKind::LParen)?.span.to_pos();

            let init = match self.check(TokenKind::Semi) {
                true => Some(self.parse_expr()?),
                false => None
            };
            let semi1 = self.expect(TokenKind::Semi)?.span.to_pos();
            let cond = match self.check(TokenKind::Semi) {
                true => Some(self.parse_expr()?),
                false => None
            };
            let semi2 = self.expect(TokenKind::Semi)?.span.to_pos();
            let step = match self.check(TokenKind::RParen) {
                true => Some(self.parse_expr()?),
                false => None
            };
            let r = self.expect(TokenKind::RParen)?.span.to_pos();
            let body = self.parse_stmt(true)?;

            StmtKind::For { for_span, l, init, semi1, cond, semi2, step, r, body }
        } else {
            unreachable!()
        };

        Ok(kind)
    }

    fn parse_jump_stmt(&mut self) -> ParserResult<StmtKind> {
        let kind = if let Some(goto_token) = self.consume_keyword(Keyword::Goto) {
            // goto label;
            let goto_span = goto_token.span;
            let ident = self.expect_ident()?;
            let span = ident.span;
            let symbol = ident.kind.into_ident().unwrap();
            let ident = Ident { span, symbol };
            let semi = self.expect(TokenKind::Semi)?.span.to_pos();

            StmtKind::Goto { goto_span, ident, semi }
        } else if let Some(continue_token) = self.consume_keyword(Keyword::Continue) {
            // continue;
            let continue_span = continue_token.span;
            let semi = self.expect(TokenKind::Semi)?.span.to_pos();
            StmtKind::Continue { continue_span, semi }
        } else if let Some(break_token) = self.consume_keyword(Keyword::Break) {
            // break;
            let break_span = break_token.span;
            let semi = self.expect(TokenKind::Semi)?.span.to_pos();
            StmtKind::Break { break_span, semi }
        } else if let Some(return_token) = self.consume_keyword(Keyword::Return) {
            // return ;
            let return_span = return_token.span;
            let expr= match self.check(TokenKind::Semi) {
                true => None,
                false => Some(self.parse_expr()?)
            };
            let semi = self.expect(TokenKind::Semi)?.span.to_pos();
            StmtKind::Return { return_span, expr , semi }
        } else {
            unreachable!()
        };

        Ok(kind)
    }


}