use super::super::ast::*;
use super::Sema;

impl Sema {
    pub(crate) fn integer_literal(&self, raw: &str) -> Option<(i128, CType)> {
        let s = raw.trim_end_matches(['u', 'U', 'l', 'L']);
        let value = if s.starts_with("0x") || s.starts_with("0X") {
            i128::from_str_radix(&s[2..], 16).ok()
        } else if s.len() > 1 && s.starts_with('0') {
            i128::from_str_radix(&s[1..], 8).ok()
        } else {
            s.parse().ok()
        }?;
        let suffix = raw[s.len()..].to_ascii_lowercase();
        if !matches!(
            suffix.as_str(),
            "" | "u" | "l" | "ul" | "lu" | "ll" | "ull" | "llu"
        ) {
            return None;
        }
        let decimal =
            !(s.starts_with("0x") || s.starts_with("0X") || s.len() > 1 && s.starts_with('0'));
        let candidates: &[(u8, bool)] = match suffix.as_str() {
            "u" => &[(3, false), (4, false), (5, false)],
            "l" if decimal => &[(4, true), (5, true)],
            "l" => &[(4, true), (4, false), (5, true), (5, false)],
            "ul" | "lu" => &[(4, false), (5, false)],
            "ll" if decimal => &[(5, true)],
            "ll" => &[(5, true), (5, false)],
            "ull" | "llu" => &[(5, false)],
            "" if decimal => &[(3, true), (4, true), (5, true)],
            "" => &[
                (3, true),
                (3, false),
                (4, true),
                (4, false),
                (5, true),
                (5, false),
            ],
            _ => return None,
        };
        let (rank, signed) = candidates.iter().copied().find(|(rank, signed)| {
            let bits = if *rank == 3 { 32 } else { 64 };
            if *signed {
                value < (1i128 << (bits - 1))
            } else {
                value < (1i128 << bits)
            }
        })?;
        let ty = CType::new(match rank {
            3 => TypeKind::Int { signed },
            4 => TypeKind::Long { signed },
            _ => TypeKind::LongLong { signed },
        });
        Some((value, ty))
    }
    pub(crate) fn decode_char(&self, s: &str) -> Option<i64> {
        let mut cs = s.chars();
        let c = cs.next()?;
        if c != '\\' {
            return Some(c as i64);
        }
        Some(match cs.next()? {
            'n' => 10,
            'r' => 13,
            't' => 9,
            '0' => 0,
            '\\' => 92,
            '\'' => 39,
            '"' => 34,
            x => x as i64,
        })
    }
    pub(crate) fn const_int(&self, e: &Expression) -> Option<i128> {
        use BinaryOp::*;
        match &e.kind {
            ExpressionKind::Integer(x) => Some(*x),
            ExpressionKind::Identifier(name) => self
                .constants
                .iter()
                .rev()
                .find_map(|scope| scope.get(name).copied()),
            ExpressionKind::Character { value, .. } => Some(*value as i128),
            ExpressionKind::Unary { op, operand } => {
                let x = self.const_int(operand)?;
                Some(match op {
                    UnaryOp::Plus => x,
                    UnaryOp::Minus => -x,
                    UnaryOp::LogicalNot => (x == 0) as i128,
                    UnaryOp::BitNot => !x,
                    _ => return None,
                })
            }
            ExpressionKind::Binary { op, left, right } => {
                let a = self.const_int(left)?;
                if *op == LogicalAnd && a == 0 {
                    return Some(0);
                }
                if *op == LogicalOr && a != 0 {
                    return Some(1);
                }
                let b = self.const_int(right)?;
                Some(match op {
                    Multiply => a * b,
                    Divide => a.checked_div(b)?,
                    Remainder => a.checked_rem(b)?,
                    Add => a + b,
                    Subtract => a - b,
                    ShiftLeft => a << b,
                    ShiftRight => a >> b,
                    Less => (a < b) as i128,
                    LessEqual => (a <= b) as i128,
                    Greater => (a > b) as i128,
                    GreaterEqual => (a >= b) as i128,
                    Equal => (a == b) as i128,
                    NotEqual => (a != b) as i128,
                    BitAnd => a & b,
                    BitXor => a ^ b,
                    BitOr => a | b,
                    LogicalAnd => (a != 0 && b != 0) as i128,
                    LogicalOr => (a != 0 || b != 0) as i128,
                })
            }
            ExpressionKind::Conditional {
                condition,
                then_expr,
                else_expr,
            } => {
                if self.const_int(condition)? != 0 {
                    self.const_int(then_expr)
                } else {
                    self.const_int(else_expr)
                }
            }
            ExpressionKind::Cast { expression, .. } => self.const_int(expression),
            ExpressionKind::SizeofType(t) => Some(self.sizeof(t) as i128),
            ExpressionKind::SizeofExpression(x) => Some(self.sizeof(&x.ty) as i128),
            ExpressionKind::Alignof(t) => Some(self.alignof(t) as i128),
            _ => None,
        }
    }
}
