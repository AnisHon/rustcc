pub const INIT_STATE: usize = 0;

static CHAR_CLASS: [(u32, u32); 9] = [
    (0, 42), (43, 43), (44, 47), (48, 48), (49, 57), (58, 96), (97, 97), (98, 122), (123, 1114111), 
];

static CHAR_CLASS_MAP: [usize; 9] = [
    0, 1, 0, 2, 3, 0, 4, 5, 0, 
];

static ASCII_MAPPING: [usize; 128] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 2, 2, 2, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 
    5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6, 7, 
    7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 
];

static BASE: [Option<usize>; 4] = [
    Some(0), None, Some(6), Some(2), 
];

static NEXT: [Option<usize>; 10] = [
    0, 1, 2, 2, 3, 3, 3, 3, 2, 2, 
];

static CHECK: [Option<usize>; 10] = [
    None, Some(0), Some(0), Some(0), Some(0), Some(0), Some(3), Some(3), Some(2), Some(2), 
];

static ROW_ID: [usize; 4] = [
    0, 1, 2, 3, 
];

///
/// get next state
/// # returns
/// -`Option<usize>`
///     - `None`: no translation, Error(in classic DFA)
///     - `Some`: next state
pub fn find_next(state_id: usize, chr: char) -> Option<usize> {
    let row_id = ROW_ID[state_id];
    let class_id = find_char(chr);
    let base = BASE[row_id];
    if base.is_none() {
        return None
    }

    let idx = base.unwrap() + class_id;
    let check = CHECK[idx];
    if check.is_none() {
        return None
    }


    if check.unwrap() == row_id {
        Some(NEXT[idx])
    } else {
        None
    }
}

fn find_char(chr: char) -> usize {
    let idx = find_idx(chr);
    CHAR_CLASS_MAP[idx]
}

fn find_idx(chr: char) -> usize {
    if chr.is_ascii() {
        // fast path
        ASCII_MAPPING[chr as usize]
    } else {
        // slow path
        binary_search(chr as u32)
    }
}

/// 二分查找
fn binary_search(chr: u32) -> usize {
    let ranges = &CHAR_CLASS;
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

    // not in ranges
    unreachable!("Not Find {}", chr)
}