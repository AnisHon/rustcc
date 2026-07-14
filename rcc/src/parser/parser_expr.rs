use super::parser_core::{PResult, Parser};
use crate::err::{Diagnostic, ErrorKind};
use crate::lex::token::{Keyword, Literal, StringEncoding, TokenKind};
use crate::parser::ast::*;
use crate::source::SourceRange;

impl Parser {
    pub(crate) fn expression(&mut self) -> PResult<Expression> {
        let first = self.assignment_expression()?;
        if self.eat(&TokenKind::Comma).is_none() {
            return Ok(first);
        }
        let start = first.range;
        let mut xs = vec![first];
        loop {
            xs.push(self.assignment_expression()?);
            if self.eat(&TokenKind::Comma).is_none() {
                break;
            }
        }
        let ty = xs.last().unwrap().ty.clone();
        let span = start.join(xs.last().unwrap().range);
        Ok(Expression {
            kind: ExpressionKind::Comma(xs),
            ty,
            category: ValueCategory::RValue,
            range: span,
        })
    }
    pub(crate) fn assignment_expression(&mut self) -> PResult<Expression> {
        let left = self.conditional_expression()?;
        let op = match self.peek().kind {
            TokenKind::Assign => AssignOp::Assign,
            TokenKind::PlusEq => AssignOp::Add,
            TokenKind::MinusEq => AssignOp::Subtract,
            TokenKind::StarEq => AssignOp::Multiply,
            TokenKind::SlashEq => AssignOp::Divide,
            TokenKind::PercentEq => AssignOp::Remainder,
            TokenKind::ShlEq => AssignOp::ShiftLeft,
            TokenKind::ShrEq => AssignOp::ShiftRight,
            TokenKind::AmpEq => AssignOp::BitAnd,
            TokenKind::CaretEq => AssignOp::BitXor,
            TokenKind::PipeEq => AssignOp::BitOr,
            _ => return Ok(left),
        };
        self.bump();
        if left.category != ValueCategory::LValue || left.ty.qualifiers.is_const {
            return Err(self.sema_err("assignment requires a modifiable lvalue", left.range));
        }
        let right = self.assignment_expression()?;
        let right = self.sema.assignment_conversion(&left.ty, right)?;
        let span = left.range.join(right.range);
        let ty = left.ty.clone();
        Ok(Expression {
            kind: ExpressionKind::Assignment {
                op,
                left: Box::new(left),
                right: Box::new(right),
            },
            ty,
            category: ValueCategory::RValue,
            range: span,
        })
    }
    pub(crate) fn conditional_expression(&mut self) -> PResult<Expression> {
        let c = self.binary_expression(1)?;
        if self.eat(&TokenKind::Question).is_none() {
            return Ok(c);
        }
        let c = self.sema.scalar_conversion(c, "conditional operand")?;
        let a = self.expression()?;
        self.expect(&TokenKind::Colon)?;
        let b = self.conditional_expression()?;
        let (a, b, ty) = self.sema.conditional_conversion(a, b)?;
        let span = c.range.join(b.range);
        Ok(Expression {
            kind: ExpressionKind::Conditional {
                condition: Box::new(c),
                then_expr: Box::new(a),
                else_expr: Box::new(b),
            },
            ty,
            category: ValueCategory::RValue,
            range: span,
        })
    }

    pub(crate) fn binary_info(k: &TokenKind) -> Option<(u8, BinaryOp)> {
        use BinaryOp::*;
        use TokenKind::*;
        Some(match k {
            Or => (1, LogicalOr),
            And => (2, LogicalAnd),
            Pipe => (3, BitOr),
            Caret => (4, BitXor),
            Amp => (5, BitAnd),
            Eq => (6, Equal),
            Ne => (6, NotEqual),
            Lt => (7, Less),
            Le => (7, LessEqual),
            Gt => (7, Greater),
            Ge => (7, GreaterEqual),
            Shl => (8, ShiftLeft),
            Shr => (8, ShiftRight),
            Plus => (9, Add),
            Minus => (9, Subtract),
            Star => (10, Multiply),
            Slash => (10, Divide),
            Percent => (10, Remainder),
            _ => return None,
        })
    }
    pub(crate) fn binary_expression(&mut self, min: u8) -> PResult<Expression> {
        let mut lhs = self.cast_expression()?;
        while let Some((prec, op)) = Self::binary_info(&self.peek().kind) {
            if prec < min {
                break;
            }
            self.bump();
            let rhs = self.binary_expression(prec + 1)?;
            lhs = self.sema.make_binary(op, lhs, rhs)?
        }
        Ok(lhs)
    }
    pub(crate) fn cast_expression(&mut self) -> PResult<Expression> {
        if self.at(&TokenKind::LParen) {
            let save = self.pos;
            let l = self.bump();
            if self.is_type_start()
                && let Ok(ty) = self.type_name()
                && self.eat(&TokenKind::RParen).is_some()
            {
                if self.at(&TokenKind::LBrace) {
                    let init = self.initializer()?;
                    let span = l.range.join(self.previous().range);
                    return Ok(Expression {
                        kind: ExpressionKind::CompoundLiteral {
                            ty: ty.clone(),
                            initializer: Box::new(init),
                        },
                        ty,
                        category: ValueCategory::LValue,
                        range: span,
                    });
                }
                let e = self.cast_expression()?;
                if matches!(ty.kind, TypeKind::Array { .. } | TypeKind::Function { .. }) {
                    return Err(self.sema_err("cast target must be a scalar type", l.range));
                }
                let span = l.range.join(e.range);
                return Ok(Expression {
                    kind: ExpressionKind::Cast {
                        target: ty.clone(),
                        expression: Box::new(e),
                    },
                    ty,
                    category: ValueCategory::RValue,
                    range: span,
                });
            }
            self.pos = save;
        }
        self.unary_expression()
    }
    pub(crate) fn unary_expression(&mut self) -> PResult<Expression> {
        let start = self.peek().range;
        if self.eat_kw(Keyword::Sizeof).is_some() {
            if self.eat(&TokenKind::LParen).is_some() {
                let save = self.pos;
                if self.is_type_start() {
                    let ty = self.type_name()?;
                    let r = self.expect(&TokenKind::RParen)?;
                    return Ok(Expression {
                        kind: ExpressionKind::SizeofType(ty),
                        ty: CType::new(TypeKind::Long { signed: false }),
                        category: ValueCategory::RValue,
                        range: start.join(r.range),
                    });
                }
                self.pos = save;
                let e = self.expression()?;
                let r = self.expect(&TokenKind::RParen)?;
                return Ok(Expression {
                    kind: ExpressionKind::SizeofExpression(Box::new(e)),
                    ty: CType::new(TypeKind::Long { signed: false }),
                    category: ValueCategory::RValue,
                    range: start.join(r.range),
                });
            }
            let e = self.unary_expression()?;
            let span = start.join(e.range);
            return Ok(Expression {
                kind: ExpressionKind::SizeofExpression(Box::new(e)),
                ty: CType::new(TypeKind::Long { signed: false }),
                category: ValueCategory::RValue,
                range: span,
            });
        }
        if self.eat_kw(Keyword::Alignof).is_some() {
            self.expect(&TokenKind::LParen)?;
            let ty = self.type_name()?;
            let r = self.expect(&TokenKind::RParen)?;
            return Ok(Expression {
                kind: ExpressionKind::Alignof(ty),
                ty: CType::new(TypeKind::Long { signed: false }),
                category: ValueCategory::RValue,
                range: start.join(r.range),
            });
        }
        let op = match self.peek().kind {
            TokenKind::Plus => Some(UnaryOp::Plus),
            TokenKind::Minus => Some(UnaryOp::Minus),
            TokenKind::Bang => Some(UnaryOp::LogicalNot),
            TokenKind::Tilde => Some(UnaryOp::BitNot),
            TokenKind::Amp => Some(UnaryOp::AddressOf),
            TokenKind::Star => Some(UnaryOp::Dereference),
            TokenKind::Inc => Some(UnaryOp::PreIncrement),
            TokenKind::Dec => Some(UnaryOp::PreDecrement),
            _ => None,
        };
        if let Some(op) = op {
            self.bump();
            let mut e = self.cast_expression()?;
            let (ty, cat) = match op {
                UnaryOp::AddressOf => {
                    if e.category == ValueCategory::RValue {
                        return Err(
                            self.sema_err("address-of requires an lvalue or function", e.range)
                        );
                    }
                    (CType::pointer(e.ty.clone()), ValueCategory::RValue)
                }
                UnaryOp::Dereference => {
                    e = self.sema.default_conversion(e);
                    match e.ty.kind.clone() {
                        TypeKind::Pointer(to) => {
                            let c = if matches!(to.kind, TypeKind::Function { .. }) {
                                ValueCategory::Function
                            } else {
                                ValueCategory::LValue
                            };
                            (*to, c)
                        }
                        _ => return Err(self.sema_err("dereference requires a pointer", e.range)),
                    }
                }
                UnaryOp::LogicalNot => {
                    e = self.sema.scalar_conversion(e, "logical not")?;
                    (CType::int(), ValueCategory::RValue)
                }
                UnaryOp::BitNot => {
                    if !e.ty.is_integer() {
                        return Err(self.sema_err("bitwise not requires integer operand", e.range));
                    }
                    e = self.sema.integer_promotion(e);
                    (e.ty.clone(), ValueCategory::RValue)
                }
                UnaryOp::Plus | UnaryOp::Minus => {
                    if !e.ty.is_arithmetic() {
                        return Err(self.sema_err(
                            "unary arithmetic operator requires arithmetic operand",
                            e.range,
                        ));
                    }
                    e = self.sema.default_conversion(e);
                    if e.ty.is_integer() {
                        e = self.sema.integer_promotion(e);
                    }
                    (e.ty.clone(), ValueCategory::RValue)
                }
                UnaryOp::PreIncrement | UnaryOp::PreDecrement => {
                    if e.category != ValueCategory::LValue
                        || !e.ty.is_scalar()
                        || e.ty.qualifiers.is_const
                    {
                        return Err(
                            self.sema_err("increment requires modifiable scalar lvalue", e.range)
                        );
                    }
                    (e.ty.clone(), ValueCategory::RValue)
                }
            };
            let span = start.join(e.range);
            return Ok(Expression {
                kind: ExpressionKind::Unary {
                    op,
                    operand: Box::new(e),
                },
                ty,
                category: cat,
                range: span,
            });
        }
        self.postfix_expression()
    }
    pub(crate) fn postfix_expression(&mut self) -> PResult<Expression> {
        let mut e = self.primary_expression()?;
        loop {
            if self.eat(&TokenKind::LBracket).is_some() {
                let i = self.expression()?;
                let r = self.expect(&TokenKind::RBracket)?;
                let operand_range = e.range.join(i.range);
                let span = e.range.join(r.range);
                let mut base_expression = self.sema.default_conversion(e);
                let mut index_expression = self.sema.default_conversion(i);
                let base = base_expression.ty.clone();
                let ity = index_expression.ty.clone();
                let elem = match (&base.kind, &ity.kind) {
                    (TypeKind::Pointer(x), _) if ity.is_integer() => (**x).clone(),
                    (_, TypeKind::Pointer(x)) if base.is_integer() => {
                        std::mem::swap(&mut base_expression, &mut index_expression);
                        (**x).clone()
                    }
                    _ => {
                        return Err(self
                            .sema_err("subscript requires a pointer and integer", operand_range));
                    }
                };
                e = Expression {
                    kind: ExpressionKind::Subscript {
                        base: Box::new(base_expression),
                        index: Box::new(self.sema.integer_promotion(index_expression)),
                    },
                    ty: elem,
                    category: ValueCategory::LValue,
                    range: span,
                };
                continue;
            }
            if self.eat(&TokenKind::LParen).is_some() {
                let mut args = vec![];
                if !self.at(&TokenKind::RParen) {
                    loop {
                        args.push(self.assignment_expression()?);
                        if self.eat(&TokenKind::Comma).is_none() {
                            break;
                        }
                    }
                }
                let r = self.expect(&TokenKind::RParen)?;
                e = self.sema.default_conversion(e);
                let fty = e.ty.clone();
                let TypeKind::Pointer(target) = fty.kind else {
                    return Err(self.sema_err("called object is not a function", e.range));
                };
                let TypeKind::Function {
                    return_type,
                    params,
                    variadic,
                    has_prototype,
                } = target.kind
                else {
                    return Err(self.sema_err("called object is not a function", e.range));
                };
                if has_prototype
                    && (args.len() < params.len() || (!variadic && args.len() != params.len()))
                {
                    return Err(self.sema_err(
                        format!(
                            "function expects {} argument(s), got {}",
                            params.len(),
                            args.len()
                        ),
                        e.range.join(r.range),
                    ));
                }
                if has_prototype {
                    for (parameter, argument) in params.iter().zip(args.iter_mut()) {
                        *argument = self
                            .sema
                            .assignment_conversion(&parameter.ty, argument.clone())?;
                    }
                }
                let span = e.range.join(r.range);
                e = Expression {
                    kind: ExpressionKind::Call {
                        callee: Box::new(e),
                        arguments: args,
                    },
                    ty: *return_type,
                    category: ValueCategory::RValue,
                    range: span,
                };
                continue;
            }
            let indirect = if self.eat(&TokenKind::Dot).is_some() {
                false
            } else if self.eat(&TokenKind::Arrow).is_some() {
                true
            } else {
                if self.eat(&TokenKind::Inc).is_some() || self.eat(&TokenKind::Dec).is_some() {
                    let dec = matches!(self.previous().kind, TokenKind::Dec);
                    if e.category != ValueCategory::LValue
                        || !e.ty.is_scalar()
                        || e.ty.qualifiers.is_const
                    {
                        return Err(self.sema_err(
                            "postfix increment requires modifiable scalar lvalue",
                            e.range,
                        ));
                    }
                    let span = e.range.join(self.previous().range);
                    let ty = e.ty.clone();
                    e = Expression {
                        kind: ExpressionKind::PostIncrement {
                            operand: Box::new(e),
                            decrement: dec,
                        },
                        ty,
                        category: ValueCategory::RValue,
                        range: span,
                    };
                    continue;
                }
                break;
            };
            let t = self.bump();
            let TokenKind::Identifier(field) = t.kind else {
                return Err(self.err("expected member name"));
            };
            let record_ty = if indirect {
                match e.ty.decay().kind {
                    TypeKind::Pointer(x) => *x,
                    _ => {
                        return Err(
                            self.sema_err("arrow requires pointer to struct or union", e.range)
                        );
                    }
                }
            } else {
                e.ty.clone()
            };
            let fields = match record_ty.kind {
                TypeKind::Struct {
                    fields: Some(f), ..
                }
                | TypeKind::Union {
                    fields: Some(f), ..
                } => f,
                _ => {
                    return Err(
                        self.sema_err("member access requires a complete struct or union", e.range)
                    );
                }
            };
            let f = fields
                .iter()
                .find(|x| x.name.as_deref() == Some(&field))
                .ok_or_else(|| self.sema_err(format!("no member named '{field}'"), t.range))?;
            let span = e.range.join(t.range);
            e = Expression {
                kind: ExpressionKind::Member {
                    base: Box::new(e),
                    field,
                    indirect,
                },
                ty: f.ty.clone(),
                category: ValueCategory::LValue,
                range: span,
            };
        }
        Ok(e)
    }
    pub(crate) fn primary_expression(&mut self) -> PResult<Expression> {
        let t = self.bump();
        match t.kind {
            TokenKind::Identifier(n) => {
                let binding = self.sema.lookup(&n).ok_or_else(|| {
                    self.sema_err(format!("use of undeclared identifier '{n}'"), t.range)
                })?;
                let ty = binding.ty;
                let category = binding.category;
                Ok(Expression {
                    kind: ExpressionKind::Identifier {
                        name: n,
                        declaration: binding.declaration,
                    },
                    ty,
                    category,
                    range: t.range,
                })
            }
            TokenKind::Literal(Literal::Integer(raw)) => {
                let (v, ty) = self
                    .sema
                    .integer_literal(&raw)
                    .ok_or_else(|| self.sema_err("invalid integer literal", t.range))?;
                Ok(Expression {
                    kind: ExpressionKind::Integer(v),
                    ty,
                    category: ValueCategory::RValue,
                    range: t.range,
                })
            }
            TokenKind::Literal(Literal::Floating(raw)) => {
                let clean = raw.trim_end_matches(['f', 'F', 'l', 'L']);
                let v = if clean.starts_with("0x") || clean.starts_with("0X") {
                    parse_hex_float(clean)
                } else {
                    clean.parse().ok()
                }
                .ok_or_else(|| self.sema_err("invalid floating literal", t.range))?;
                let ty = if raw.ends_with(['f', 'F']) {
                    CType::new(TypeKind::Float)
                } else if raw.ends_with(['l', 'L']) {
                    CType::new(TypeKind::LongDouble)
                } else {
                    CType::new(TypeKind::Double)
                };
                Ok(Expression {
                    kind: ExpressionKind::Floating(v),
                    ty,
                    category: ValueCategory::RValue,
                    range: t.range,
                })
            }
            TokenKind::Literal(Literal::Character {
                value: raw,
                encoding,
            }) => {
                let v = self
                    .sema
                    .decode_char(&raw)
                    .ok_or_else(|| self.sema_err("invalid character literal", t.range))?;
                let ty = match encoding {
                    StringEncoding::Narrow | StringEncoding::Wide => CType::int(),
                    StringEncoding::Utf16 => CType::new(TypeKind::Short { signed: false }),
                    StringEncoding::Utf32 => CType::uint(),
                    StringEncoding::Utf8 => {
                        return Err(
                            self.sema_err("u8 character constants are not part of C11", t.range)
                        );
                    }
                };
                Ok(Expression {
                    kind: ExpressionKind::Character { value: v, encoding },
                    ty,
                    category: ValueCategory::RValue,
                    range: t.range,
                })
            }
            TokenKind::Literal(Literal::String {
                value: mut raw,
                mut encoding,
            }) => {
                let mut span = t.range;
                while let TokenKind::Literal(Literal::String {
                    value: s,
                    encoding: next_encoding,
                }) = self.peek().kind.clone()
                {
                    span = span.join(self.bump().range);
                    if encoding == StringEncoding::Narrow {
                        encoding = next_encoding;
                    } else if next_encoding != StringEncoding::Narrow && next_encoding != encoding {
                        return Err(
                            self.sema_err("incompatible adjacent string literal encodings", span)
                        );
                    }
                    raw.push_str(&s)
                }
                let element = match encoding {
                    StringEncoding::Narrow | StringEncoding::Utf8 => {
                        CType::new(TypeKind::Char { signed: None })
                    }
                    StringEncoding::Utf16 => CType::new(TypeKind::Short { signed: false }),
                    StringEncoding::Utf32 => CType::uint(),
                    StringEncoding::Wide => CType::int(),
                };
                let ty = CType::new(TypeKind::Array {
                    element: Box::new(element),
                    size: ArraySize::Constant(raw.chars().count() + 1),
                });
                Ok(Expression {
                    kind: ExpressionKind::String {
                        value: raw,
                        encoding,
                    },
                    ty,
                    category: ValueCategory::LValue,
                    range: span,
                })
            }
            TokenKind::LParen => {
                let e = self.expression()?;
                self.expect(&TokenKind::RParen)?;
                Ok(e)
            }
            TokenKind::Keyword(Keyword::Generic) => self.generic_selection(t.range),
            _ => Err(Diagnostic::new(
                ErrorKind::Syntax,
                "expected expression",
                t.range,
            )),
        }
    }
    pub(crate) fn generic_selection(&mut self, start: SourceRange) -> PResult<Expression> {
        self.expect(&TokenKind::LParen)?;
        let controlling = self.assignment_expression()?;
        let controlling = self.sema.default_conversion(controlling);
        self.expect(&TokenKind::Comma)?;
        let mut selected = None;
        let mut default = None;
        loop {
            if self.eat_kw(Keyword::Default).is_some() {
                self.expect(&TokenKind::Colon)?;
                default = Some(self.assignment_expression()?)
            } else {
                let ty = self.type_name()?;
                self.expect(&TokenKind::Colon)?;
                let e = self.assignment_expression()?;
                if self.sema.compatible(&controlling.ty.decay(), &ty) {
                    if selected.is_some() {
                        return Err(
                            self.sema_err("multiple matching generic associations", e.range)
                        );
                    }
                    selected = Some(e)
                }
            }
            if self.eat(&TokenKind::Comma).is_none() {
                break;
            }
        }
        let r = self.expect(&TokenKind::RParen)?;
        let choice = selected.or(default).ok_or_else(|| {
            self.sema_err(
                "generic selection has no matching association",
                controlling.range,
            )
        })?;
        let ty = choice.ty.clone();
        let category = choice.category;
        Ok(Expression {
            kind: ExpressionKind::GenericSelection {
                controlling: Box::new(controlling),
                selected: Box::new(choice),
            },
            ty,
            category,
            range: start.join(r.range),
        })
    }
}
fn parse_hex_float(raw: &str) -> Option<f64> {
    let raw = raw.strip_prefix("0x").or_else(|| raw.strip_prefix("0X"))?;
    let (mantissa, exponent) = raw.split_once(['p', 'P'])?;
    let exponent: i32 = exponent.parse().ok()?;
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let whole = if whole.is_empty() {
        0.0
    } else {
        u128::from_str_radix(whole, 16).ok()? as f64
    };
    let mut scale = 1.0 / 16.0;
    let mut value = whole;
    for digit in fraction.chars() {
        value += digit.to_digit(16)? as f64 * scale;
        scale /= 16.0;
    }
    Some(value * 2f64.powi(exponent))
}
