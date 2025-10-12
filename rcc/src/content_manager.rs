use std::ops::Range;
use std::str::Chars;

///
/// 内容管理
/// # Members
/// - `content`: 代码
/// - `line_ranges`: 每个行对应的区间，索引+1是行号
///
pub struct ContentManager {
    content: String,
    line_ranges: Vec<(usize, usize)>,
}

impl ContentManager {
    pub fn new(content: String) -> ContentManager {
        let mut line_ranges = Vec::new();
        let mut beg = 0;
        for line in content.lines() {
            let end = beg + line.len(); // 计算行结束偏移
            line_ranges.push((beg, end)); // 索引 + 1 就是行号
            beg = end; // 作为下一行开始偏移
        }

        Self {
            content,
            line_ranges,
        }
    }

    /// [beg, end)
    pub fn str(&self, range: Range<usize>) -> &str {
        &self.content[range]
    }

    pub fn chars(&self, pos: usize) -> Chars {
        self.content[pos..].chars()
    }

}

