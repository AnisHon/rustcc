use std::collections::{BTreeMap, BTreeSet};

///
/// 通过二分查找获取ranges的索引，通过class_map查询 索引 -> class_id
///
#[derive(Debug, Clone)]
pub struct CharClassSet {
    ranges: Vec<(u32, u32)>,
    class_map: Vec<usize>,
    ascii_mapping: [usize; 128],
}

impl CharClassSet {
    pub fn new(ranges: Vec<(u32, u32, BTreeSet<usize>)>) -> Self {
        let class_map = Self::build_class_id_table(&ranges); // 通过 tag_ids 构建 class_id表
        let ranges: Vec<_> = ranges // 转换为结构体ranges类型
            .into_iter()
            .map(|(l, r, _)| (l, r))
            .collect();
        let mut ascii_mapping: [usize; 128] = [0; 128]; // 缓存ascii范围字符
        for chr in 0..ascii_mapping.len() {
            ascii_mapping[chr] = binary_search(&ranges, chr as u32);
        }

        Self {
            ranges,
            class_map,
            ascii_mapping,
        }
    }

    pub fn size(&self) -> usize {
        self.ranges.len()
    }

    pub fn get_raw_ranges(&self) -> &Vec<(u32, u32)> {
        &self.ranges
    }

    pub fn get_raw_class_map(&self) -> &Vec<usize> {
        &self.class_map
    }

    pub fn get_raw_ascii_mapping(&self) -> &[usize; 128] {
        &self.ascii_mapping
    }

    fn build_class_id_table(ranges: &Vec<(u32, u32, BTreeSet<usize>)>) -> Vec<usize> {
        let mut tag_class_map: BTreeMap<Vec<usize>, usize> = BTreeMap::new();
        let mut next_class_id = 0;
        let mut class_map = Vec::new();
        class_map.reserve(ranges.len());

        // HashSet不能Hash所以这里转换成Vec
        let ranges: Vec<_> = ranges
            .iter()
            .map(|(l, r, tags)| {
                let mut tags: Vec<usize> = tags.into_iter().map(|x| *x).collect();
                tags.sort_unstable();
                (l, r, tags)
            })
            .collect();

        for (_, _, tag_ids) in ranges.iter() {
            let id = match tag_class_map.get(tag_ids) {
                // 没有get or default?
                None => {
                    tag_class_map.insert(tag_ids.clone(), next_class_id);
                    let copy = next_class_id;
                    next_class_id += 1;
                    copy
                }
                Some(x) => *x,
            };
            class_map.push(id);
        }

        class_map
    }

    fn find_idx(&self, chr: char) -> usize {
        if chr.is_ascii() {
            // 快速路径
            self.ascii_mapping[chr as usize]
        } else {
            // 慢路径
            binary_search(&self.ranges, chr as u32) // 出错会触发panic
        }
    }

    /// 查询字符
    pub fn find_char(&self, chr: char) -> usize {
        let idx = self.find_idx(chr);
        self.class_map[idx]
    }

    /// 查询区间 [begin, end]
    pub fn find_interval(&self, begin: char, end: char) -> BTreeSet<usize> {
        let begin_idx = self.find_idx(begin);
        let end_idx = self.find_idx(end);
        let mut interval = BTreeSet::new(); // class_id相对离散，无法连续表示，但是好在范围足够小
        // 一般区间查询都是供给内部使用，应当都是切分点，否则一定出错
        assert_eq!(begin as u32, self.ranges[begin_idx].0);
        assert_eq!(end as u32, self.ranges[end_idx].1);

        interval.extend(&self.class_map[begin_idx..=end_idx]); // [begin_idx, end_idx]

        interval
    }

    /// 查询翻转区间 全集U U - [begin, end]
    pub fn find_reverse_interval(&self, begin: char, end: char) -> BTreeSet<usize> {
        let begin_idx = self.find_idx(begin);
        let end_idx = self.find_idx(end);
        let mut interval = BTreeSet::new();
        assert_eq!(begin as u32, self.ranges[begin_idx].0);
        assert_eq!(end as u32, self.ranges[end_idx].1);
        
        interval.extend(&self.class_map[..begin_idx]); // [0, begin_idx)
        interval.extend(&self.class_map[end_idx + 1..]); // (end_idx, max]

        interval
    }
}

/// 二分查找
pub fn binary_search(ranges: &Vec<(u32, u32)>, chr: u32) -> usize {
    let mut idx_left = 0;
    let mut idx_right = ranges.len() - 1; // 左闭右闭

    while idx_left <= idx_right {
        let idx_mid = (idx_left + idx_right) / 2;
        let (l, r) = ranges[idx_mid];

        if l <= chr && chr <= r {
            return idx_mid;
        }

        if l < chr {
            idx_left = idx_mid + 1;
        } else {
            idx_right = idx_mid - 1;
        }
    }

    // 在我的设计下，会有一个覆盖全集的大范围，不会出现这种情况
    panic!("Not Find {}", chr)
}
