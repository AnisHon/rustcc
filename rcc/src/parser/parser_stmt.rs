use super::parser_core::{PResult, Parser};
use crate::lex::token::{Keyword, TokenKind};
use crate::parser::ast::*;

impl Parser {
    pub(crate) fn compound_statement(&mut self, create_scope: bool) -> PResult<Statement> {
        let l = self.expect(&TokenKind::LBrace)?;
        if create_scope {
            self.sema.enter_scope()
        }
        let mut items = vec![];
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
            if self.kw(Keyword::StaticAssert) {
                items.push(BlockItem::StaticAssert(self.static_assert()?));
                continue;
            }
            if self.is_declaration_start() {
                for d in self.local_declaration()? {
                    items.push(BlockItem::Declaration(d))
                }
            } else {
                items.push(BlockItem::Statement(self.statement()?))
            }
        }
        let r = self.expect(&TokenKind::RBrace)?;
        if create_scope {
            self.sema.leave_scope()
        }
        Ok(Statement {
            kind: StatementKind::Compound(items),
            span: l.span.join(r.span),
        })
    }
    pub(crate) fn is_declaration_start(&self) -> bool {
        self.is_type_start()
            || matches!(
                self.peek().kind,
                TokenKind::Keyword(
                    Keyword::Typedef
                        | Keyword::Extern
                        | Keyword::Static
                        | Keyword::Auto
                        | Keyword::Register
                        | Keyword::ThreadLocal
                        | Keyword::Alignas
                        | Keyword::Inline
                        | Keyword::Noreturn
                )
            )
    }
    pub(crate) fn statement(&mut self) -> PResult<Statement> {
        let start = self.peek().span;
        if self.at(&TokenKind::LBrace) {
            return self.compound_statement(true);
        }
        if self.eat(&TokenKind::Semi).is_some() {
            return Ok(Statement {
                kind: StatementKind::Empty,
                span: start.join(self.previous().span),
            });
        }
        if self.eat_kw(Keyword::If).is_some() {
            self.expect(&TokenKind::LParen)?;
            let c = self.expression()?;
            self.sema.require_scalar(&c, "if condition")?;
            self.expect(&TokenKind::RParen)?;
            let then_branch = Box::new(self.statement()?);
            let else_branch = if self.eat_kw(Keyword::Else).is_some() {
                Some(Box::new(self.statement()?))
            } else {
                None
            };
            let end = else_branch.as_ref().map_or(then_branch.span, |x| x.span);
            return Ok(Statement {
                kind: StatementKind::If {
                    condition: c,
                    then_branch,
                    else_branch,
                },
                span: start.join(end),
            });
        }
        if self.eat_kw(Keyword::Switch).is_some() {
            self.expect(&TokenKind::LParen)?;
            let e = self.expression()?;
            if !e.ty.is_integer() {
                return Err(self.sema_err("switch expression must have integer type", e.span));
            }
            self.expect(&TokenKind::RParen)?;
            self.sema.begin_switch();
            let body = Box::new(self.statement()?);
            self.sema.end_switch();
            let span = start.join(body.span);
            return Ok(Statement {
                kind: StatementKind::Switch {
                    expression: e,
                    body,
                },
                span,
            });
        }
        if self.eat_kw(Keyword::While).is_some() {
            self.expect(&TokenKind::LParen)?;
            let c = self.expression()?;
            self.sema.require_scalar(&c, "while condition")?;
            self.expect(&TokenKind::RParen)?;
            self.sema.begin_loop();
            let body = Box::new(self.statement()?);
            self.sema.end_loop();
            let span = start.join(body.span);
            return Ok(Statement {
                kind: StatementKind::While { condition: c, body },
                span,
            });
        }
        if self.eat_kw(Keyword::Do).is_some() {
            self.sema.begin_loop();
            let body = Box::new(self.statement()?);
            self.sema.end_loop();
            self.eat_kw(Keyword::While)
                .ok_or_else(|| self.err("expected while after do body"))?;
            self.expect(&TokenKind::LParen)?;
            let c = self.expression()?;
            self.sema.require_scalar(&c, "do-while condition")?;
            self.expect(&TokenKind::RParen)?;
            let semi = self.expect(&TokenKind::Semi)?;
            return Ok(Statement {
                kind: StatementKind::DoWhile { body, condition: c },
                span: start.join(semi.span),
            });
        }
        if self.eat_kw(Keyword::For).is_some() {
            self.expect(&TokenKind::LParen)?;
            self.sema.enter_scope();
            let init = if self.is_declaration_start() {
                ForInit::Declaration(self.local_declaration()?)
            } else {
                let e = if self.at(&TokenKind::Semi) {
                    None
                } else {
                    Some(self.expression()?)
                };
                self.expect(&TokenKind::Semi)?;
                ForInit::Expression(e)
            };
            let condition = if self.at(&TokenKind::Semi) {
                None
            } else {
                let e = self.expression()?;
                self.sema.require_scalar(&e, "for condition")?;
                Some(e)
            };
            self.expect(&TokenKind::Semi)?;
            let step = if self.at(&TokenKind::RParen) {
                None
            } else {
                Some(self.expression()?)
            };
            self.expect(&TokenKind::RParen)?;
            self.sema.begin_loop();
            let body = Box::new(self.statement()?);
            self.sema.end_loop();
            self.sema.leave_scope();
            let span = start.join(body.span);
            return Ok(Statement {
                kind: StatementKind::For {
                    init,
                    condition,
                    step,
                    body,
                },
                span,
            });
        }
        if self.eat_kw(Keyword::Goto).is_some() {
            let t = self.bump();
            let TokenKind::Identifier(n) = t.kind else {
                return Err(self.err("expected label after goto"));
            };
            let s = self.expect(&TokenKind::Semi)?;
            return Ok(Statement {
                kind: StatementKind::Goto(n),
                span: start.join(s.span),
            });
        }
        if self.eat_kw(Keyword::Continue).is_some() {
            if !self.sema.in_loop() {
                return Err(self.sema_err("continue is only valid in a loop", start));
            }
            let s = self.expect(&TokenKind::Semi)?;
            return Ok(Statement {
                kind: StatementKind::Continue,
                span: start.join(s.span),
            });
        }
        if self.eat_kw(Keyword::Break).is_some() {
            if !self.sema.in_loop() && !self.sema.in_switch() {
                return Err(self.sema_err("break is only valid in a loop or switch", start));
            }
            let s = self.expect(&TokenKind::Semi)?;
            return Ok(Statement {
                kind: StatementKind::Break,
                span: start.join(s.span),
            });
        }
        if self.eat_kw(Keyword::Return).is_some() {
            let e = if self.at(&TokenKind::Semi) {
                None
            } else {
                Some(self.expression()?)
            };
            let s = self.expect(&TokenKind::Semi)?;
            let ret = self
                .sema
                .current_return()
                .ok_or_else(|| self.sema_err("return outside a function", start))?;
            match (&ret.kind, &e) {
                (TypeKind::Void, None) => {}
                (TypeKind::Void, Some(x)) => {
                    return Err(self.sema_err("void function cannot return a value", x.span));
                }
                (_, None) => {
                    return Err(self.sema_err("non-void function must return a value", start));
                }
                (_, Some(x)) => self.sema.require_assignable(&ret, x)?,
            }
            return Ok(Statement {
                kind: StatementKind::Return(e),
                span: start.join(s.span),
            });
        }
        if self.eat_kw(Keyword::Case).is_some() {
            if !self.sema.in_switch() {
                return Err(self.sema_err("case outside switch", start));
            }
            let e = self.assignment_expression()?;
            if self.sema.const_int(&e).is_none() {
                return Err(self.sema_err("case value must be an integer constant", e.span));
            }
            self.expect(&TokenKind::Colon)?;
            let st = Box::new(self.statement()?);
            let span = start.join(st.span);
            return Ok(Statement {
                kind: StatementKind::Case {
                    value: e,
                    statement: st,
                },
                span,
            });
        }
        if self.eat_kw(Keyword::Default).is_some() {
            if !self.sema.in_switch() {
                return Err(self.sema_err("default outside switch", start));
            }
            self.expect(&TokenKind::Colon)?;
            let st = Box::new(self.statement()?);
            let span = start.join(st.span);
            return Ok(Statement {
                kind: StatementKind::Default { statement: st },
                span,
            });
        }
        if let TokenKind::Identifier(n) = self.peek().kind.clone()
            && self
                .tokens
                .get(self.pos + 1)
                .is_some_and(|t| matches!(t.kind, TokenKind::Colon))
        {
            self.bump();
            self.bump();
            let st = Box::new(self.statement()?);
            let span = start.join(st.span);
            return Ok(Statement {
                kind: StatementKind::Label {
                    name: n,
                    statement: st,
                },
                span,
            });
        }
        let e = self.expression()?;
        let semi = self.expect(&TokenKind::Semi)?;
        Ok(Statement {
            kind: StatementKind::Expression(e),
            span: start.join(semi.span),
        })
    }
}
