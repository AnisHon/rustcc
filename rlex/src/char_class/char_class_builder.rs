use crate::char_class::char_class_set::CharClassSet;
use crate::parser::ast::ASTNode;
use std::collections::{HashSet, VecDeque};

pub struct CharClassBuilder {
    total_set: (u32, u32), // 字符集 全集比如 0-127 比如 0-0x10FFFF
}

impl CharClassBuilder {
    pub fn new(total_set: (u32, u32)) -> Self {
        Self { total_set }
    }

    pub fn build_char_class_set(&self, ast: &ASTNode) -> CharClassSet {
        let mut ranges = get_ranges(ast);
        ranges.insert(self.total_set); // 添加一个 unicode 全集
        let ranges = build_range(ranges);
        CharClassSet::new(ranges)
    }
}

/// 用于区间扫描算法
#[derive(Debug)]
enum Status {
    Begin,
    End,
}

///
/// 通过线性扫描搭配活跃标签划分等价类
///
/// ### 算法描述
/// 假设区间[a,z] [b,x] [x-z]
/// ```text
/// a----------------z (1)
///   b-----------x    (2)
///               x--z (3)
/// ```
/// 1. 如果指针在a，则激活a的开始标签，active = {1}
/// 2. 指针移动到b，则发生切分[a, a]，此时 active = {1, 2}
/// 3. 指针移动到x，切分[b,x - 1]，遇到3的起始点，切分[x, x] active = {1, 2, 3}
/// 4. 指针移动到z，切分[x + 1, z]，遇到1 3的终点 active = {}
///
/// ### 区间查询
/// 最后会得到多个可能完全等价的等价区间，在区间查询时可能会比较吃力，但是因为本身就是利用原区间切分的，因此所有区间查询一定是分割端点
/// 这意味着[l, r]一定会完全覆盖多个等价类区间，不可能出现错位情况
/// 1. 假设区间[l, r]可以通过二分查找到l的区间和r的索引计作a, b
/// 2. 维护一个表记录每个索引对应的class_id，这个class_id是通过active集合计算的，active集合相同则class_id相同
/// 3. 遍历a..=b 得到多个class_id 返回Vec<usize>
fn build_range(ranges: HashSet<(u32, u32)>) -> Vec<(u32, u32, HashSet<usize>)> {
    if ranges.is_empty() {
        return Vec::new();
    }
    // [(l/r, status, tag_id), ...]，pointes采用左闭右开区间
    let mut points: Vec<(u32, Status, usize)> = Vec::new();
    let mut active_tag = HashSet::new();
    let mut results = Vec::new();
    for (i, (l, r)) in ranges.into_iter().enumerate() {
        let tag_id = i;
        points.push((l, Status::Begin, tag_id));
        points.push((r + 1, Status::End, tag_id));
    }

    points.sort_by_key(|(l, _, _)| *l);

    let mut last = points[0].0; // 从第二个开始，
    active_tag.insert(points[0].2); // 推入第一个状态
    for (p, status, tag_id) in points.into_iter().skip(1) {
        if p != last {
            // 如果p == last [p, last)为空集
            results.push((last, p - 1, active_tag.clone()));
            last = p;
        }

        match status {
            Status::Begin => active_tag.insert(tag_id),
            Status::End => active_tag.remove(&tag_id),
        };
    }
    results
}

/// 迭代遍历节点获取所有出现的字符
fn get_ranges(ast: &ASTNode) -> HashSet<(u32, u32)> {
    let mut ranges: HashSet<(u32, u32)> = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(ast);

    while !queue.is_empty() {
        let top = queue.pop_front().unwrap();

        match top {
            ASTNode::Literal(chr) => {
                let chr = *chr as u32;
                ranges.insert((chr, chr)); // 单个字符的区间
            }
            ASTNode::CharClass(node) => {
                node.chars.iter().for_each(|x| {
                    let x = *x as u32;
                    ranges.insert((x, x));
                });
                node.ranges.iter().for_each(|x| {
                    let (l, r) = x;
                    ranges.insert((*l as u32, *r as u32));
                });
            }
            ASTNode::Star(node)
            | ASTNode::Question(node)
            | ASTNode::Plus(node)
            | ASTNode::Range(node, _) => {
                queue.push_back(node);
            }

            ASTNode::Concatenation(nodes) | ASTNode::Alternation(nodes) => {
                nodes.iter().for_each(|x| {
                    queue.push_back(x);
                });
            }
        }
    }

    ranges
}
