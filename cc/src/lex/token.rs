use crate::lex::lex_yy::TokenType;

/// 词法分析输出Token
#[derive(Debug, Clone)]
pub struct Token {
    pub pos: usize,
    pub line: usize,
    pub typ: TokenType,
    pub value: String,
} 