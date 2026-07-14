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
        let lookup = |name: &str| {
            self.constants
                .iter()
                .rev()
                .find_map(|scope| scope.get(name).copied())
        };
        crate::ConstantEvaluator::new(&self.target, &lookup)
            .evaluate_integer(e)
            .ok()
    }
}
