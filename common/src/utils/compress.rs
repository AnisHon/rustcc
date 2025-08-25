use bitvec::bitvec;
use bitvec::vec::BitVec;
use crate::lex::ClassID;

pub fn compress_matrix<T: Eq + Copy>(
    matrix: &Vec<Vec<T>>, zero_value: T
) -> (Vec<Option<usize>>, Vec<T>, Vec<Option<usize>>) {
    let rows = matrix.len();
    let stride = matrix[0].len();
    let mut bitmap = bitvec![0; rows];

    let mut base: Vec<Option<usize>> = vec![None; rows];
    let mut next: Vec<T> = vec![zero_value; stride];
    let mut check: Vec<Option<usize>> = vec![None; rows];

    let mut row_cols: Vec<(_, Vec<_>)> = (0..rows)
        .map(|row|
            (
                row,
                matrix[row].iter().enumerate().filter_map(|(idx, &x)| match x.ne(&zero_value) {
                    true => Some(idx),
                    false => None
                }).collect()
            )
        )
        .collect();
    row_cols.sort_by_key(|(_, edges)| edges.len());
    row_cols.reverse();


    for (row, cols) in row_cols {
        if cols.is_empty() {
            continue;
        }

        // 最大转移，内存分配上界
        let max_row = cols.iter().max().copied().unwrap();

        let offset = alloc_base(&mut bitmap, &cols, max_row);

        let max_size = offset + max_row + 1;

        if max_size > next.len() {
            next.resize(max_size, zero_value);
            check.resize(max_size, None);
        }

        for edge in cols {
            let pos = offset + edge;
            next[pos] = matrix[row][edge];
            check[pos] = Some(row);
        }
        base[row] = Some(offset);
    }

    (base, next, check)
}

// 分配一个可用的base
fn alloc_base(bitmap: &mut BitVec, edges: &Vec<ClassID>, max_edge: usize) -> usize {

    let mut offset: usize = match bitmap.first_zero() { // 查询初始位置
        Some(x) => x,
        None => {
            bitmap.resize(bitmap.len() + max_edge, false);
            bitmap.first_zero().unwrap()
        }
    };

    while try_allocate(bitmap, edges, offset, max_edge) {
        offset += 1;
    }

    for &edge in edges {
        let pos = offset + edge;
        bitmap.set(pos, true);
    }


    offset

}

fn try_allocate(bitmap: &mut BitVec, edges: &Vec<ClassID>, offset: usize, max_edge: usize) -> bool {
    for &edge in edges.iter() {
        let pos = edge + offset;
        // 自动增长逻辑
        if pos >= bitmap.len() {
            bitmap.resize(pos + max_edge, false);
        }

        let occupied = bitmap[pos];

        // 被占用无法分配
        if occupied {
            return false;
        }
    }

    // 一切畅通可以分配
    true
}