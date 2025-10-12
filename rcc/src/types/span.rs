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
}

impl Debug for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..={}", self.start, self.end)
    }
}

///
/// 对于列表 比如`a, b, c` `a; b; c;`
/// 该结构负责存储符号位置和列表本身
///
#[derive(Debug, Clone)]
pub struct SepList<T> {
    pub list: Vec<T>,
    pub sep: Vec<Span>,
}


impl<T> Default for SepList<T> {
    fn default() -> Self {
        Self { list: Vec::new(), sep: Vec::new() }
    }
}


pub trait UnwrapSpan {
    fn unwrap_span(&self) -> Span;
}