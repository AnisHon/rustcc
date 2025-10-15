use std::fmt::{Debug, Formatter};

///
/// 节点对应的位置区间
///
#[derive(Default, Clone, PartialEq, Eq, Hash, Copy)]
pub struct Span {
    pub start: usize, // 在源文件中的字节偏移
    pub end: usize,   // 在源文件中的字节偏移（不包含end）
}
impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        assert!(start <= end);
        Span { start, end }
    }
    
    pub fn span(lo: Span, hi: Span) -> Self {
        let start = lo.start;
        let end = hi.end;
        assert!(start <= end);
        Span { start, end }
    }
    

    /// merge计算得到新的span
    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// 直接merge到自己
    pub fn merge_self(&mut self, other: &Span) {
        self.start = self.start.min(other.start);
        self.end = self.end.max(other.end);
    }

    pub fn to_pos(&self) -> Pos {
        assert_eq!(self.end - self.start, 1);
        Pos { pos: self.start }
    }
}

impl Debug for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..={}", self.start, self.end)
    }
}

///
/// 对于单字符情况下，使用span浪费空间
///
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct Pos {
    pub pos: usize,
}

impl Pos {
    pub fn new(pos: usize) -> Self {
        Self { pos }
    }
}