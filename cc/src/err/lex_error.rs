use std::error::Error;
use std::fmt::{Display, Formatter};

pub type LexResult<T> = Result<T, LexError>;

#[derive(Debug)]
pub struct LexError {
    pub pos: usize,
    pub line: usize,
    pub msg: String,    // 错误信息
    pub content: String, // 错误位置
    
}

impl LexError {
    pub fn new(pos: usize, line: usize, msg: &str, content: String) -> Self {
        Self { pos, line, msg: msg.to_string(), content }
    }
    
}

impl Display for LexError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for LexError {
    
}

