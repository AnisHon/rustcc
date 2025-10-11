use std::arch::aarch64::vqrdmulhs_lane_s32;
use crate::err::parser_error;
use crate::err::parser_error::{ParserError, ParserResult};
use crate::lex::types::token::Token;
use crate::lex::types::token_kind::{Keyword, LiteralKind, TokenKind};
use crate::lex::types::token_kind::TokenKind::Comma;
use crate::parser::parser_core::Parser;
use crate::types::span::Span;

impl Parser {
    
    fn check_string(&self) -> bool {
        match &self.stream.peek().kind {
            TokenKind::Literal(x) => !matches!(x, LiteralKind::String { .. }),
            _ => false,
        }
    }
    
    fn consume_constant(&mut self) -> Option<Token> {
        let is_constant = match &self.stream.peek().kind {
            TokenKind::Literal(x) => !matches!(x, LiteralKind::String { .. }),
            _ => false,
        };
        self.next_conditional(is_constant)
    }
    
    fn consume_string(&mut self) -> Option<Token> {
        let is_string = self.check_string();
        self.next_conditional(is_string)
    }

    fn consume_unary_op(&mut self) -> Option<Token> {
        use TokenKind::*;
        let is_unary_op =
            matches!(self.stream.peek().kind, Amp | Star | Plus | Minus | Tilde | Bang);

        self.next_conditional(is_unary_op)
    }

    fn consume_assign_op(&mut self) -> Option<Token> {
        use TokenKind::*;
        let is_assign_op =
            matches!(self.stream.peek().kind,
                Assign | StarEq | SlashEq | PercentEq
                | PlusEq | MinusEq| ShlEq | ShrEq| AmpEq
                | CaretEq | PipeEq
            );
        self.next_conditional(is_assign_op)
    }

    fn parse_string(&mut self) -> Vec<Token> {
        let mut strings = Vec::with_capacity(1);
        while let Some(string) = self.consume_string() {
            strings.push(string)
        }
        strings
    }


    fn parse_primary(&mut self) -> ParserResult<()> {
        let lo = self.stream.span();
        if self.check_ident() {
            let ident = self.stream.next();
        } else if let Some(constant) = self.consume_constant() {
            todo!()
        } else if self.check_string() {
            let strings = self.parse_string();
            todo!()
        } else if let Some(lparen) = self.consume(TokenKind::LParen) {
            let expr = self.parse_expr()?;
            let rparen = self.expect(TokenKind::RParen)?;
        } else {
            // 匹配失败，无法恢复，报错
            let kind = parser_error::ErrorKind::Expect {
                expect: "identifier, integer, float, char, string, '('".to_owned()
            };
            return Err(self.error_here(kind));
        }
        let hi = self.stream.prev_span();

        Ok(())
    }

    fn parse_postfix(&mut self) -> ParserResult<()> {
        let lo = self.stream.span();
        let expr = self.parse_primary()?;

        loop {
            if let Some(lparen) = self.consume(TokenKind::LBracket) {
                // 数组访问[]
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RBracket)?;
            } else if let Some(lparen) = self.consume(TokenKind::LParen) {
                // 函数调用()
                self.parse_sep_list(TokenKind::Comma, TokenKind::RParen, Self::parse_assign_expr)?;
                let rparen = self.expect(TokenKind::RParen)?;

            } else if let Some(lparen) = self.consume(TokenKind::Dot) {
                // 成员访问 a.b
                let ident = self.expect_ident()?;
                
            } else if let Some(lparen) = self.consume(TokenKind::Arrow) {
                // 成员访问 a->b
                let ident = self.expect_ident()?;

            } else if let Some(lparen) = self.consume(TokenKind::Inc) {


            } else if let Some(lparen) = self.consume(TokenKind::Dec) {

            } else {
                break
            }
        }
        let hi = self.stream.prev_span();
        Ok(expr)
    }

    fn parse_unary(&mut self) -> ParserResult<()> {
        let lo = self.stream.span();
        if let Some(inc) = self.consume(TokenKind::Inc) {
            // 前置++
            let expr = self.parse_unary()?;

        } else if let Some(dec) = self.consume(TokenKind::Dec) {
            // 前置--
            let expr = self.parse_unary()?;
        } else if let Some(op) = self.consume_unary_op() {
            // 一元运算符
            let expr = self.parse_cast()?;
        } else if let Some(sizeof) = self.consume_keyword(Keyword::Sizeof) {
            // sizeof
            if let Some(lparen) = self.consume(TokenKind::LParen) {
                // sizeof typename
                let type_name = self.parse_type_name()?;
                let rparen = self.expect(TokenKind::RParen)?;
            } else {

                let expr = self.parse_unary()?;

            }
        } else {
            // 什么都不是
            return self.parse_postfix();
        }
        let hi = self.stream.prev_span();

        Ok(())
    }

    fn parse_cast(&mut self) -> ParserResult<()> {
        let lo = self.stream.span();
        if let Some(lparen) =self.consume(TokenKind::LParen) {
            let type_name = self.parse_type_name()?;
            let rparen = self.expect(TokenKind::RParen)?;
            let expr = self.parse_cast();
        } else {
            return self.parse_unary();
        }
        let hi = self.stream.prev_span();

        Ok(())
    }


    /// 处理multiplicative-expression的{ ("*" | "/" | "%") cast-expression }*部分
    fn parse_multiplicative_rhs(&mut self, lhs: (), lo: Span) -> ParserResult<()> {
        use TokenKind::*;
        if let Some(token)  = self.consumes(&[Star, Slash, Percent]) {
            let expr = self.parse_cast()?;
            let hi = self.stream.prev_span();
            return self.parse_multiplicative_rhs((), lo);
        }
        Ok(lhs)
    }

    /// multiplicative-expression
    fn parse_multiplicative(&mut self) -> ParserResult<()> {
        let lo = self.stream.span();
        let expr = self.parse_cast()?;
        let result = self.parse_multiplicative_rhs(expr, lo)?;
        Ok(result)
    }

    fn parse_additive_rhs(&mut self, lhs: (), lo: Span) -> ParserResult<()> {
        use TokenKind::*;
        if let Some(token) =self.consumes(&[Plus, Minus]) {
            let expr = self.parse_multiplicative()?;
            let hi = self.stream.prev_span();
            return self.parse_additive_rhs((), lo);
        }
        Ok(lhs)
    }

    /// additive-expression
    fn parse_additive(&mut self) -> ParserResult<()> {
        let lo = self.stream.span();
        let expr = self.parse_multiplicative()?;
        let result = self.parse_additive_rhs(expr, lo)?;
        Ok(result)
    }

    fn parse_shift_rhs(&mut self, lhs: (), lo: Span) -> ParserResult<()> {
        use TokenKind::*;
        if let Some(token) =self.consumes(&[Shl, Shr]) {
            let expr = self.parse_additive()?;
            let hi = self.stream.prev_span();
            return self.parse_shift_rhs((), lo);
        }
        Ok(lhs)
    }

    fn parse_shift(&mut self) -> ParserResult<()> {
        let lo = self.stream.span();
        let expr = self.parse_multiplicative()?;
        let result = self.parse_additive_rhs(expr, lo)?;
        Ok(result)
    }

    fn parse_relational_rhs(&mut self, lhs: (), lo: Span) -> ParserResult<()> {
        use TokenKind::*;
        if let Some(token) =self.consumes(&[Lt, Gt, Le, Ge]) {
            let expr = self.parse_shift()?;
            let hi = self.stream.prev_span();
            return self.parse_relational_rhs((), lo);
        }
        Ok(lhs)
    }

    fn parse_relational(&mut self) -> ParserResult<()> {
        let lo = self.stream.span();
        let expr = self.parse_shift()?;
        let result = self.parse_relational_rhs(expr, lo)?;
        Ok(result)
    }

    fn parse_equality_rhs(&mut self, lhs: (), lo: Span) -> ParserResult<()> {
        use TokenKind::*;
        if let Some(token) =self.consumes(&[Eq, Ne]) {
            let expr = self.parse_relational()?;
            let hi = self.stream.prev_span();
            return self.parse_equality_rhs((), lo);
        }
        Ok(lhs)
    }

    fn parse_equality(&mut self) -> ParserResult<()> {
        let lo = self.stream.span();
        let expr = self.parse_relational()?;
        let result = self.parse_equality_rhs(expr, lo)?;
        Ok(result)
    }

    fn parse_and_rhs(&mut self, lhs: (), lo: Span) -> ParserResult<()> {
        use TokenKind::*;
        if let Some(token) =self.consumes(&[Amp]) {
            let expr = self.parse_equality()?;
            let hi = self.stream.prev_span();
            return self.parse_and_rhs((), lo);
        }
        Ok(lhs)
    }

    fn parse_and(&mut self) -> ParserResult<()> {
        let lo = self.stream.span();
        let expr = self.parse_equality()?;
        let result = self.parse_and_rhs(expr, lo)?;
        Ok(result)
    }


    fn parse_exclusive_or_rhs(&mut self, lhs: (), lo: Span) -> ParserResult<()> {
        use TokenKind::*;
        if let Some(token) =self.consumes(&[Caret]) {
            let expr = self.parse_and()?;
            let hi = self.stream.prev_span();
            return self.parse_exclusive_or_rhs((), lo);
        }
        Ok(lhs)
    }

    fn parse_exclusive_or(&mut self) -> ParserResult<()> {
        let lo = self.stream.span();
        let expr = self.parse_and()?;
        let result = self.parse_exclusive_or_rhs(expr, lo)?;
        Ok(result)
    }

    fn parse_inclusive_or_rhs(&mut self, lhs: (), lo: Span) -> ParserResult<()> {
        use TokenKind::*;
        if let Some(token) =self.consumes(&[Amp]) {
            let expr = self.parse_exclusive_or()?;
            let hi = self.stream.prev_span();
            return self.parse_inclusive_or_rhs((), lo);
        }
        Ok(lhs)
    }

    fn parse_inclusive_or(&mut self) -> ParserResult<()> {
        let lo = self.stream.span();
        let expr = self.parse_exclusive_or()?;
        let result = self.parse_inclusive_or_rhs(expr, lo)?;
        Ok(result)
    }


    fn parse_logical_and_rhs(&mut self, lhs: (), lo: Span) -> ParserResult<()> {
        use TokenKind::*;
        if let Some(token) =self.consumes(&[Amp]) {
            let expr = self.parse_inclusive_or()?;
            let hi = self.stream.prev_span();
            return self.parse_logical_and_rhs((), lo);
        }
        Ok(lhs)
    }

    fn parse_logical_and(&mut self) -> ParserResult<()> {
        let lo = self.stream.span();
        let expr = self.parse_inclusive_or()?;
        let result = self.parse_logical_and_rhs(expr, lo)?;
        Ok(result)
    }

    fn parse_logical_or_rhs(&mut self, lhs: (), lo: Span) -> ParserResult<()> {
        use TokenKind::*;
        if let Some(token) =self.consumes(&[Amp]) {
            let expr = self.parse_logical_and()?;
            let hi = self.stream.prev_span();
            return self.parse_logical_or_rhs((), lo);
        }
        Ok(lhs)
    }

    fn parse_logical_or(&mut self) -> ParserResult<()> {
        let lo = self.stream.span();
        let expr = self.parse_logical_and()?;
        let result = self.parse_logical_or_rhs(expr, lo)?;
        Ok(result)
    }


    fn parse_conditional(&mut self) -> ParserResult<()> {
        let lo = self.stream.span();
        let cond = self.parse_logical_or()?;

        let question = self.consume(TokenKind::Question);

        // 不是三元表达式
        if question.is_none() {
            return Ok(cond);
        }
        // 一定是三元表达式
        let question = question.unwrap();
        let expr1 = self.parse_expr()?;
        let colon = self.expect(TokenKind::Colon)?; // 必须有 ':'
        let expr2 = self.parse_conditional()?;
        let hi = self.stream.prev_span();

        Ok(())
    }


    pub(crate) fn parse_assign_expr(&mut self) -> ParserResult<()> {
        let lo = self.stream.span();
        let lhs = self.parse_conditional()?;
        let assign_op = self.consume_assign_op();
        if assign_op.is_none() {
            return Ok(lhs); // 不是赋值表达式
        }
        // todo 检查表达式是不是unary expr
        let assign_op = assign_op.unwrap();
        let rhs = self.parse_assign_expr()?;
        let hi = self.stream.prev_span();
        Ok(())
    }

    pub(crate) fn parse_expr_rhs(&mut self, lhs: (), lo: Span) -> ParserResult<()> {
        if let Some(comma) = self.consume(Comma) {
            let expr = self.parse_assign_expr()?;
            let hi = self.stream.prev_span();
            return self.parse_expr_rhs((), lo);
        }
        Ok(lhs)
    }

    pub(crate) fn parse_expr(&mut self) -> ParserResult<()> {
        let lo = self.stream.span();
        let expr = self.parse_assign_expr()?;
        let result = self.parse_expr_rhs(expr, lo)?;

        Ok(result)
    }


    
    
}