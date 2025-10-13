use crate::err::parser_error;
use crate::err::parser_error::{ParserResult};
use crate::lex::types::token::Token;
use crate::lex::types::token_kind::{Keyword, LiteralKind, TokenKind};
use crate::parser::parser_core::Parser;
use crate::parser::types::ast::expr::{Expr, ExprKind, Parameter};
use crate::types::span::Span;

impl Parser {
    
    fn check_string(&self) -> bool {
        match &self.stream.peek().kind {
            TokenKind::Literal(x) => matches!(x, LiteralKind::String { .. }),
            _ => false,
        }
    }


    fn next_is_type_name(&self) -> bool {
        let token = self.stream.peek_next();
        self.is_type_qual(token) || self.is_type_spec(token)
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


    fn parse_primary_expr(&mut self) -> ParserResult<Box<Expr>> {
        let lo = self.stream.span();
        let kind = if let Some(ident) = self.consume_ident() { // ident
            ExprKind::make_decl_ref(ident)
        } else if let Some(constant) = self.consume_constant() { // constant
            ExprKind::make_literal(constant)
        } else if self.check_string() { // string
            let strings = self.parse_string();
            ExprKind::make_string(strings)
        } else if let Some(lparen) = self.consume(TokenKind::LParen) { // ( expr )
            let expr = self.parse_expr()?;
            let rparen = self.expect(TokenKind::RParen)?;
            ExprKind::make_paren(lparen, expr, rparen)
        } else {
            // 匹配失败，无法恢复，报错
            println!("error: {:?}", self.stream.peek());
            let kind = parser_error::ErrorKind::Expect {
                expect: "identifier, integer, float, char, string, '('".to_owned()
            };
            let error = self.error_here(kind);
            panic!("{error}");
            return Err(error);
        };
        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let expr = Expr::new_box(kind, span);
        Ok(expr)
    }

    fn parse_postfix_expr_suffix(&mut self, mut lhs: Box<Expr>) -> ParserResult<Box<Expr>> {
        use TokenKind::*;
        let lo = self.stream.span();
        loop {
            let kind = if let Some(lparen) = self.consume(LBracket) {
                // 数组访问[]
                let index = self.parse_expr()?;
                let rparen = self.expect(RBracket)?;
                ExprKind::make_index(lhs, lparen, index, rparen)
            } else if let Some(lparen) = self.consume(LParen) {
                // 函数调用()
                let param = self.parse_expr_list()?;
                let rparen = self.expect(RParen)?;
                ExprKind::make_call(lhs, lparen, param, rparen)
            } else if let Some(dot) = self.consume(Dot) {
                // 成员访问 a.b
                let ident = self.expect_ident()?;
                let field = ident.kind.into_ident().unwrap();
                ExprKind::make_dot(lhs, dot, field)
            } else if let Some(arrow) = self.consume(Arrow) {
                // 成员访问 a->b
                let ident = self.expect_ident()?;
                let field = ident.kind.into_ident().unwrap();
                ExprKind::make_dot(lhs, arrow, field)
            } else if let Some(op) = self.consumes(&[Inc, Dec]) {
                ExprKind::make_post(lhs, op)
            } else {
                break;
            };
            let hi = self.stream.prev_span();
            let span = Span::span(lo, hi);
            lhs = Expr::new_box(kind, span);
        }

        Ok(lhs)
    }

    fn parse_postfix_expr(&mut self) -> ParserResult<Box<Expr>> {
        let expr = self.parse_primary_expr()?;
        let expr = self.parse_postfix_expr_suffix(expr)?;
        Ok(expr)
    }

    fn parse_expr_list(&mut self) -> ParserResult<Parameter> {
        let mut param = Parameter::new();
        if self.check(TokenKind::RParen) {
            return Ok(param);
        }

        loop {
            let expr = self.parse_assign_expr()?;
            param.exprs.push(expr);
            if let Some(comma) = self.consume(TokenKind::Comma) {
                param.commas.push(comma.span)
            } else if self.check(TokenKind::RParen) {
                break
            } else {
                let kind = parser_error::ErrorKind::Expect { expect: "expression".to_owned() };
                return Err(self.error_here(kind))
            }
        }
        Ok(param)
    }

    fn parse_unary_expr(&mut self) -> ParserResult<Box<Expr>> {
        let lo = self.stream.span();
        let kind = if let Some(op) = self.consume_pair(TokenKind::Inc, TokenKind::Dec) {
            // 前置++
            let expr = self.parse_unary_expr()?;
            ExprKind::make_pre(op, expr)
        } else if let Some(op) = self.consume_unary_op() {
            // 一元运算符
            let expr = self.parse_cast_expr()?;
            ExprKind::make_unary(op, expr)
        } else if let Some(sizeof) = self.consume_keyword(Keyword::Sizeof) {
            // sizeof
            if let Some(lparen) = self.consume(TokenKind::LParen) {
                // sizeof typename
                let type_name = self.parse_type_name()?;
                let rparen = self.expect(TokenKind::RParen)?;
                ExprKind::make_size_of_type(sizeof, lparen, type_name, rparen)
            } else {
                let expr = self.parse_unary_expr()?;
                ExprKind::make_size_of_expr(sizeof, expr)
            }
        } else {
            // 什么都不是
            return self.parse_postfix_expr();
        };
        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let expr = Expr::new_box(kind, span);
        Ok(expr)
    }

    fn parse_cast_expr(&mut self) -> ParserResult<Box<Expr>> {
        let lo = self.stream.span();
        let kind = if self.check(TokenKind::LParen) && self.next_is_type_name() {
            let lparen = self.stream.next();
            let type_name = self.parse_type_name()?;
            let rparen = self.expect(TokenKind::RParen)?;
            let expr = self.parse_cast_expr()?;
            ExprKind::make_cast(lparen, type_name, rparen, expr)
        } else {
            return self.parse_unary_expr();
        };
        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let expr = Expr::new_box(kind, span);
        Ok(expr)
    }


    /// 处理multiplicative-expression的{ ("*" | "/" | "%") cast-expression }*部分
    fn parse_multiplicative_expr_rhs(&mut self, lhs: Box<Expr>, lo: Span) -> ParserResult<Box<Expr>> {
        use TokenKind::*;
        if let Some(op)  = self.consumes(&[Star, Slash, Percent]) {
            let rhs = self.parse_cast_expr()?;
            let hi = self.stream.prev_span();
            let span = Span::span(lo, hi);

            let kind = ExprKind::make_binary(lhs, op, rhs);
            let expr = Expr::new_box(kind, span);

            return self.parse_multiplicative_expr_rhs(expr, lo);
        }
        Ok(lhs)
    }

    /// multiplicative-expression
    fn parse_multiplicative_expr(&mut self) -> ParserResult<Box<Expr>> {
        let lo = self.stream.span();
        let lhs = self.parse_cast_expr()?;
        let expr = self.parse_multiplicative_expr_rhs(lhs, lo)?;
        Ok(expr)
    }

    fn parse_additive_expr_rhs(&mut self, lhs: Box<Expr>, lo: Span) -> ParserResult<Box<Expr>> {
        use TokenKind::*;
        if let Some(op) =self.consume_pair(Plus, Minus) {
            let rhs = self.parse_multiplicative_expr()?;
            let hi = self.stream.prev_span();
            let span = Span::span(lo, hi);

            let kind = ExprKind::make_binary(lhs, op, rhs);
            let expr = Expr::new_box(kind, span);

            return self.parse_additive_expr_rhs(expr, lo);
        }
        Ok(lhs)
    }

    /// additive-expression
    fn parse_additive_expr(&mut self) -> ParserResult<Box<Expr>> {
        let lo = self.stream.span();
        let lhs = self.parse_multiplicative_expr()?;
        let expr = self.parse_additive_expr_rhs(lhs, lo)?;
        Ok(expr)
    }

    fn parse_shift_expr_rhs(&mut self, lhs: Box<Expr>, lo: Span) -> ParserResult<Box<Expr>> {
        use TokenKind::*;
        if let Some(op) =self.consume_pair(Shl, Shr) {
            let rhs = self.parse_additive_expr()?;
            let hi = self.stream.prev_span();
            let span = Span::span(lo, hi);

            let kind = ExprKind::make_binary(lhs, op, rhs);
            let expr = Expr::new_box(kind, span);

            return self.parse_shift_expr_rhs(expr, lo);
        }
        Ok(lhs)
    }

    fn parse_shift_expr(&mut self) -> ParserResult<Box<Expr>> {
        let lo = self.stream.span();
        let lhs = self.parse_additive_expr()?;
        let expr = self.parse_shift_expr_rhs(lhs, lo)?;
        Ok(expr)
    }

    fn parse_relational_expr_rhs(&mut self, lhs: Box<Expr>, lo: Span) -> ParserResult<Box<Expr>> {
        use TokenKind::*;
        if let Some(op) =self.consumes(&[Lt, Gt, Le, Ge]) {
            let rhs = self.parse_shift_expr()?;
            let hi = self.stream.prev_span();
            let span = Span::span(lo, hi);

            let kind = ExprKind::make_binary(lhs, op, rhs);
            let expr = Expr::new_box(kind, span);
            
            return self.parse_relational_expr_rhs(expr, lo);
        }
        Ok(lhs)
    }

    fn parse_relational_expr(&mut self) -> ParserResult<Box<Expr>> {
        let lo = self.stream.span();
        let lhs = self.parse_shift_expr()?;
        let expr = self.parse_relational_expr_rhs(lhs, lo)?;
        Ok(expr)
    }

    fn parse_equality_expr_rhs(&mut self, lhs: Box<Expr>, lo: Span) -> ParserResult<Box<Expr>> {
        use TokenKind::*;
        if let Some(op) =self.consume_pair(Eq, Ne) {
            let rhs = self.parse_relational_expr()?;
            let hi = self.stream.prev_span();
            let span = Span::span(lo, hi);

            let kind = ExprKind::make_binary(lhs, op, rhs);
            let expr = Expr::new_box(kind, span);
            
            return self.parse_equality_expr_rhs(expr, lo);
        }
        Ok(lhs)
    }

    fn parse_equality_expr(&mut self) -> ParserResult<Box<Expr>> {
        let lo = self.stream.span();
        let expr = self.parse_relational_expr()?;
        let result = self.parse_equality_expr_rhs(expr, lo)?;
        Ok(result)
    }

    fn parse_and_expr_rhs(&mut self, lhs: Box<Expr>, lo: Span) -> ParserResult<Box<Expr>> {
        use TokenKind::*;
        if let Some(op) =self.consume(Amp) {
            let rhs = self.parse_equality_expr()?;
            let hi = self.stream.prev_span();
            let span = Span::span(lo, hi);

            let kind = ExprKind::make_binary(lhs, op, rhs);
            let expr = Expr::new_box(kind, span);
            
            return self.parse_and_expr_rhs(expr, lo);
        }
        Ok(lhs)
    }

    fn parse_and_expr(&mut self) -> ParserResult<Box<Expr>> {
        let lo = self.stream.span();
        let lhs = self.parse_equality_expr()?;
        let expr = self.parse_and_expr_rhs(lhs, lo)?;
        Ok(expr)
    }


    fn parse_exclusive_or_expr_rhs(&mut self, lhs: Box<Expr>, lo: Span) -> ParserResult<Box<Expr>> {
        use TokenKind::*;
        if let Some(op) =self.consume(Caret) {
            let rhs = self.parse_and_expr()?;
            let hi = self.stream.prev_span();
            let span = Span::span(lo, hi);

            let kind = ExprKind::make_binary(lhs, op, rhs);
            let expr = Expr::new_box(kind, span);
            
            return self.parse_exclusive_or_expr_rhs(expr, lo);
        }
        Ok(lhs)
    }

    fn parse_exclusive_or_expr(&mut self) -> ParserResult<Box<Expr>> {
        let lo = self.stream.span();
        let lhs = self.parse_and_expr()?;
        let expr = self.parse_exclusive_or_expr_rhs(lhs, lo)?;
        Ok(expr)
    }

    fn parse_inclusive_or_expr_rhs(&mut self, lhs: Box<Expr>, lo: Span) -> ParserResult<Box<Expr>> {
        use TokenKind::*;
        if let Some(op) =self.consume(Pipe) {
            let rhs = self.parse_exclusive_or_expr()?;
            let hi = self.stream.prev_span();
            let span = Span::span(lo, hi);

            let kind = ExprKind::make_binary(lhs, op, rhs);
            let expr = Expr::new_box(kind, span);
            
            return self.parse_inclusive_or_expr_rhs(expr, lo);
        }
        Ok(lhs)
    }

    fn parse_inclusive_or_expr(&mut self) -> ParserResult<Box<Expr>> {
        let lo = self.stream.span();
        let lhs = self.parse_exclusive_or_expr()?;
        let expr = self.parse_inclusive_or_expr_rhs(lhs, lo)?;
        Ok(expr)
    }


    fn parse_logical_and_expr_rhs(&mut self, lhs: Box<Expr>, lo: Span) -> ParserResult<Box<Expr>> {
        use TokenKind::*;
        if let Some(op) =self.consume(And) {
            let rhs = self.parse_inclusive_or_expr()?;
            let hi = self.stream.prev_span();
            let span = Span::span(lo, hi);

            let kind = ExprKind::make_binary(lhs, op, rhs);
            let expr = Expr::new_box(kind, span);
            
            return self.parse_logical_and_expr_rhs(expr, lo);
        }
        Ok(lhs)
    }

    fn parse_logical_and_expr(&mut self) -> ParserResult<Box<Expr>> {
        let lo = self.stream.span();
        let lhs = self.parse_inclusive_or_expr()?;
        let expr = self.parse_logical_and_expr_rhs(lhs, lo)?;
        Ok(expr)
    }

    fn parse_logical_or_expr_rhs(&mut self, lhs: Box<Expr>, lo: Span) -> ParserResult<Box<Expr>> {
        use TokenKind::*;
        if let Some(op) =self.consume(Or) {
            let rhs = self.parse_logical_and_expr()?;
            let hi = self.stream.prev_span();
            let span = Span::span(lo, hi);

            let kind = ExprKind::make_binary(lhs, op, rhs);
            let expr = Expr::new_box(kind, span);
            
            return self.parse_logical_or_expr_rhs(expr, lo);
        }
        Ok(lhs)
    }

    fn parse_logical_or_expr(&mut self) -> ParserResult<Box<Expr>> {
        let lo = self.stream.span();
        let rhs = self.parse_logical_and_expr()?;
        let expr = self.parse_logical_or_expr_rhs(rhs, lo)?;
        Ok(expr)
    }


    fn parse_conditional_expr(&mut self) -> ParserResult<Box<Expr>> {
        let lo = self.stream.span();
        let cond = self.parse_logical_or_expr()?;

        let question = self.consume(TokenKind::Question);

        // 不是三元表达式
        if question.is_none() {
            return Ok(cond);
        }
        // 一定是三元表达式
        let question = question.unwrap();
        let then_expr = self.parse_expr()?;
        let colon = self.expect(TokenKind::Colon)?; // 必须有 ':'
        let else_expr = self.parse_conditional_expr()?;
        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);
        
        let kind = ExprKind::make_ternary(cond, question, then_expr, colon, else_expr);
        let expr = Expr::new_box(kind, span);
        
        Ok(expr)
    }


    pub(crate) fn parse_assign_expr(&mut self) -> ParserResult<Box<Expr>> {
        let lo = self.stream.span();
        let lhs = self.parse_conditional_expr()?;
        let assign_op = self.consume_assign_op();
        if assign_op.is_none() {
            return Ok(lhs); // 不是赋值表达式
        }
        
        if !lhs.is_lvalue() {
            let kind = parser_error::ErrorKind::NotAssignable { ty: "Expression".to_owned() };
            return Err(self.error_here(kind))
        }

        let assign_op = assign_op.unwrap();
        let rhs = self.parse_assign_expr()?;
        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let kind = ExprKind::make_assign(lhs, assign_op, rhs);
        let expr = Expr::new_box(kind, span);
        
        Ok(expr)
    }

    fn parse_expr_rhs(&mut self, lhs: Box<Expr>, lo: Span) -> ParserResult<Box<Expr>> {
        if let Some(op) = self.consume(TokenKind::Comma) {
            let rhs = self.parse_assign_expr()?;
            let hi = self.stream.prev_span();
            let span = Span::span(lo, hi);

            let kind = ExprKind::make_binary(lhs, op, rhs);
            let expr = Expr::new_box(kind, span);

            return self.parse_expr_rhs(expr, lo);
        }
        Ok(lhs)
    }

    pub(crate) fn parse_expr(&mut self) -> ParserResult<Box<Expr>> {
        let lo = self.stream.span();
        let lhs = self.parse_assign_expr()?;
        let expr = self.parse_expr_rhs(lhs, lo)?;
        Ok(expr)
    }
}