use crate::lex::lex_yy::TokenType;
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

    pub fn new(beg: usize, typ: TokenType, value: String ) -> Self {
        let end = beg + value.len();
        let (value, typ) = match typ {
            TokenType::Id => (TokenValue::String(value), typ as usize),
            TokenType::Hex => {
                let (value, signed) = hex2int(value);
                (TokenValue::Number{value, signed}, TokenType::Int as usize)
            },
            TokenType::Oct => {
                let (value, signed) = oct2int(value);
                (TokenValue::Number{value, signed}, TokenType::Int as usize)
            },
            TokenType::Int => {
                let (value, signed) = str2int(value);
                (TokenValue::Number{value, signed}, TokenType::Int as usize)
            },
            TokenType::Float => (TokenValue::Float(str2float(value)), typ as usize),
            TokenType::StringLiteral => (TokenValue::String(format_str(value)), typ as usize),
            TokenType::CharacterConstant => (TokenValue::Char(format_char(value)), typ as usize),
            TokenType::TypeName => (TokenValue::String(value), TokenType::Float as usize),
            _ => (TokenValue::Other, typ as usize)
        };

        Self {
            beg,
            end,
            typ,
            value,
        }
    }
    
    pub fn is(&self, typ: TokenType) -> bool {
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
    pub fn ignore(&self) -> bool {
        self.typ == TokenType::BlockComment as usize
            || self.typ == TokenType::LineComment as usize
            || self.typ == TokenType::Whitespace as usize
    }

    pub fn as_type(&self) -> Option<TokenType> {
       TokenType::from_usize(self.typ)
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.as_type() {
            None => {write!(f, "Token(END)")}
            Some(x) => {write!(f, "Token(beg: {:?}, end: {:?}, type: {:?}, value: {:?})", self.beg, self.end, x, self.value)}
        }
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











