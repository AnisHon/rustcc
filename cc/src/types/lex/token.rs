use crate::types::lex::token_kind::TokenKind;
use crate::types::span::Span;
use enum_as_inner::EnumAsInner;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Clone, EnumAsInner)]
pub enum TokenValue {
    Number{ value: usize, signed: bool },
    Float(f64),
    String(String),
    Char(u8),
    Other,
}

impl Display for TokenValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenValue::Number { value, signed } =>
                if *signed {
                    write!(f, "{:#}", value)
                } else {
                    write!(f, "{:#}u", value)
                }
            TokenValue::Float(x) => write!(f, "{:#}", x),
            TokenValue::String(x) => write!(f, "{:?}", x),
            TokenValue::Char(x) => write!(f, "{:?}", *x as char),
            TokenValue::Other => write!(f, "None"),
        }
    }
}

/// 词法分析输出Token
#[derive(Debug, Clone)]
pub struct Token {
    pub span: Span,
    pub kind: TokenKind,
    pub value: TokenValue,
}

impl Token {

    pub fn new(beg: usize, end: usize, kind: TokenKind, value: String) -> Self {
        assert!(beg <= end);
        let value = match kind {
            TokenKind::ID => TokenValue::String(value),
            TokenKind::Hex => {
                let (value, signed) = hex2int(value);
                TokenValue::Number{value, signed}
            },
            TokenKind::Oct => {
                let (value, signed) = oct2int(value);
                TokenValue::Number{value, signed}
            },
            TokenKind::Int => {
                let (value, signed) = str2int(value);
                TokenValue::Number{value, signed}
            },
            TokenKind::Float => TokenValue::Float(str2float(value)),
            TokenKind::StringLiteral => TokenValue::String(format_str(value)),
            TokenKind::CharacterConstant => TokenValue::Char(format_char(value)),
            _ => TokenValue::Other
        };

        // 统一数字类型
        let kind = match kind {
            TokenKind::Hex | TokenKind::Oct => TokenKind::Int,
            _ => kind
        };

        Self {
            span: Span::new(beg, end),
            kind,
            value,
        }
    }

}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.value {
            TokenValue::Other => {
                write!(
                    f,
                    "{:?} @{:?}",
                    self.kind,
                    self.span,
                )
            }
            _ => {
                write!(
                    f,
                    "{:?}({}) @{:?}",
                    self.kind,
                    self.value,
                    self.span,
                )

            }
        }

    }
}

// todo 处理数字后缀

fn oct2int(num: String) -> (usize, bool) {
    // todo 解析int
    let value = isize::from_str_radix(&num, 8).unwrap() as usize;
    (value, true)
}

fn hex2int(num: String) -> (usize, bool) {
    // todo 解析int
    let value = isize::from_str_radix(&num[2..], 16).unwrap() as usize; // 去除0x部分
    (value, true)
}
fn str2int(num: String) -> (usize, bool) {
    // todo 解析int
    let value = num.as_str().parse::<isize>().unwrap() as usize;
    (value, true)
}

fn str2float(num: String) -> f64 {
    // todo 解析float
    num.parse::<f64>().unwrap()
}

fn format_str(str: String) -> String {
    // todo 格式化string
    str
}

fn format_char(str: String) -> u8 {
    // todo 格式化char
    str.into_bytes()[0]
}











