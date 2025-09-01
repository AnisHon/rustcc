use std::fmt::{Debug, Formatter};
use enum_as_inner::EnumAsInner;
use num_traits::FromPrimitive;
use crate::lex::lex_yy::TokenType;
use crate::parser::parser_yy::END_SYMBOL;
use crate::types::symbol_table::SymbolTable;

#[derive(Debug, Clone, EnumAsInner)]
pub enum TokenValue {
    Number(usize),
    Float(f64),
    String(String),
    Char(char),
    Other,
}

/// 词法分析输出Token
#[derive(Clone)]
pub struct Token {
    pub pos: usize,
    pub line: usize,
    pub typ: usize,
    pub value: TokenValue,
}

impl Token {

    pub fn new(pos: usize, line: usize, typ: TokenType, value: String ) -> Self {

        let (value, typ) = match typ {
            TokenType::Id => (TokenValue::String(value), typ as usize),
            TokenType::Hex => (TokenValue::Number(hex2int(value)), TokenType::Int as usize),
            TokenType::Oct => (TokenValue::Number(oct2int(value)), TokenType::Int as usize),
            TokenType::Int => (TokenValue::Number(str2int(value)), TokenType::Int as usize),
            TokenType::Float => (TokenValue::Float(str2float(value)), typ as usize),
            TokenType::StringLiteral => (TokenValue::String(format_str(value)), typ as usize),
            TokenType::CharacterConstant => (TokenValue::Char(format_char(value)), typ as usize),
            TokenType::TypeName => (TokenValue::String(value), TokenType::Float as usize),
            _ => (TokenValue::Other, typ as usize)
        };

        Self {
            pos,
            line,
            typ,
            value,
        }
    }
    
    pub fn is(&self, typ: TokenType) -> bool {
        self.typ == typ as usize
    }
    
    pub fn end() -> Self {
        Self {
            pos: 0,
            line: 0,
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
            Some(x) => {write!(f, "Token(pos: {:?}, line: {:?}, type: {:?}, value: {:?})", self.pos, self.line, x, self.value)}
        }
    }
}

fn oct2int(num: String) -> usize {
    isize::from_str_radix(&num, 8).unwrap() as usize
}

fn hex2int(num: String) -> usize {
    isize::from_str_radix(&num[2..], 16).unwrap() as usize // 去除0x部分
}
fn str2int(num: String) -> usize {
    isize::from_str_radix(num.as_str(), 10).unwrap() as usize
}

fn str2float(num: String) -> f64 {
    num.parse::<f64>().unwrap()
}

fn format_str(str: String) -> String {
    str
}

fn format_char(str: String) -> char {
    str.chars().nth(0).unwrap()
}











