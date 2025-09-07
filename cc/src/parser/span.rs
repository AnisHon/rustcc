use crate::types::token::Token;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize, // 在源文件中的字节偏移
    pub end: usize,   // 在源文件中的字节偏移（不包含end）
}
impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        assert!(start <= end);
        Span { start, end }
    }
    
    pub fn from_token(token: &Token) -> Self {
        Span {
            start: token.beg,
            end: token.end,
        }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    pub fn merge_self(&mut self, other: &Span) {
        self.start = self.start.min(other.start);
        self.end = self.end.max(other.end);
    }
}