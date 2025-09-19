use crate::types::lex::token_kind::TokenKind;
use crate::parser::parser_yy::END_SYMBOL;
use enum_as_inner::EnumAsInner;
use num_traits::FromPrimitive;
use std::fmt::{Debug, Formatter};

#[derive(Debug, Clone, EnumAsInner)]
pub enum TokenValue {
    Number{ value: usize, signed: bool },
    Float(f64),
    String(String),
    Char(u8),
    Other,
}

/// 词法分析输出Token
#[derive(Clone)]
pub struct Token {
    pub beg: usize,
    pub end: usize,
    pub typ: usize,
    pub value: TokenValue,
}

impl Token {

    pub fn new(beg: usize, typ: usize, value: String ) -> Self {
        let end = beg + value.len();
        let typ = TokenKind::from_usize(typ).unwrap();
        let (value, typ) = match typ {
            TokenKind::ID => (TokenValue::String(value), typ as usize),
            TokenKind::Hex => {
                let (value, signed) = hex2int(value);
                (TokenValue::Number{value, signed}, TokenKind::Int as usize)
            },
            TokenKind::Oct => {
                let (value, signed) = oct2int(value);
                (TokenValue::Number{value, signed}, TokenKind::Int as usize)
            },
            TokenKind::Int => {
                let (value, signed) = str2int(value);
                (TokenValue::Number{value, signed}, TokenKind::Int as usize)
            },
            TokenKind::Float => (TokenValue::Float(str2float(value)), typ as usize),
            TokenKind::StringLiteral => (TokenValue::String(format_str(value)), typ as usize),
            TokenKind::CharacterConstant => (TokenValue::Char(format_char(value)), typ as usize),
            TokenKind::TypeName => (TokenValue::String(value), TokenKind::Float as usize),
            _ => (TokenValue::Other, typ as usize)
        };

        Self {
            beg,
            end,
            typ,
            value,
        }
    }
    
    pub fn is(&self, typ: TokenKind) -> bool {
        self.typ == typ as usize
    }
    
    pub fn end_token() -> Self {
        Self {
            beg: 0,
            end: 0,
            typ: END_SYMBOL,
            value: TokenValue::Other,
        }
    }

    pub fn as_type(&self) -> TokenKind {
        TokenKind::from_usize(self.typ).unwrap()
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f, 
            "Token(beg: {:?}, end: {:?}, type: {:?}, value: {:?})", 
            self.beg, 
            self.end, 
            self.as_type(), 
            self.value
        )
    }
}

// todo 处理数字后缀

fn oct2int(num: String) -> (usize, bool) {
    // todo 解析int
    let value = isize::from_str_radix(&num, 8).unwrap() as usize;
    (value, false)
}

fn hex2int(num: String) -> (usize, bool) {
    // todo 解析int
    let value = isize::from_str_radix(&num[2..], 16).unwrap() as usize; // 去除0x部分
    (value, false)
}
fn str2int(num: String) -> (usize, bool) {
    // todo 解析int
    let value = num.as_str().parse::<isize>().unwrap() as usize;
    (value, false)
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











