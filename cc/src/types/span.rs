use crate::types::token::Token;

///
/// 节点对应的位置区间
///
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

    ///
    /// 假设 token 数组“有序”，只读取头尾元素
    ///
    pub fn from_tokens(tokens: Vec<&Token>) -> Self {
        assert!(!tokens.is_empty());
        let first = tokens.first().unwrap().beg;
        let last = tokens.last().unwrap().end;

        Self {
            start: first,
            end: last
        }
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

///
/// 对于的被包裹类型，比如(A) \[X\]可以直接用这个结构
///
#[derive(Debug, Clone)]
pub struct Delim<T> {
    pub l: Span,
    pub inner: T,
    pub r: Span,
}

impl <T> Delim<T> {
    pub fn new(l: &Token, inner: T, r: &Token) -> Self {
        Self {
            l: Span::from_token(l),
            inner,
            r: Span::from_token(r),
        }
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

impl<T> SepList<T> {
    pub fn new(item: T) -> Self {
        Self { list: vec![item], sep: Vec::new() }
    }

    pub fn push_item(&mut self, item: T) {
        self.list.push(item);
    }

    pub fn push_sep(&mut self, sep: Span) {
        self.sep.push(sep);
    }
}

impl<T> Default for SepList<T> {
    fn default() -> Self {
        Self { list: Vec::new(), sep: Vec::new() }
    }
}


pub trait UnwrapSpan {
    fn unwrap_span(&self) -> Span;
}