use crate::rlexer::lexer::Lexer;
use bitvec::bitvec;
use bitvec::prelude::BitVec;
use common::lex::{ClassID, StateID};
use heck::ToUpperCamelCase;
use std::fs;
use tera::{Context, Tera};

const TEMPLATE: &str = include_str!("../../resources/lex.rs.tera");
pub struct LexWriter {
    path: String,
    lexer: Lexer,
}

impl LexWriter {
    pub fn new(path: String, lexer: Lexer) -> Self {
        Self { path, lexer }
    }

    pub fn write(self) {
        let mut tera = Tera::default();
        tera.add_raw_template("lex_yy.tera", TEMPLATE).unwrap();

        let dfa = self.lexer.get_dfa();
        let char_class = self.lexer.get_char_class_set();
        let lex = self.lexer.get_lex();
        let enum_name: Vec<_> = lex
            .iter()
            .map(|lex_struct| lex_struct.name.clone().to_lowercase().to_upper_camel_case())
            .collect();
        let state_map: Vec<_> = (0..dfa.size())
            .map(|idx| {
                let meta = dfa.get_meta(idx);
                if meta.terminate {
                    Some(enum_name[meta.id].clone())
                } else {
                    None
                }
            })
            .collect();

        let (base, next, check) = self.compress_dfa();


        // 准备上下文
        let mut context = Context::new();
        context.insert("init_state", &dfa.get_init_state());
        context.insert("state_map_size", &state_map.len());
        context.insert("state_map", &state_map);
        context.insert("enums", &enum_name);
        context.insert("base", &base);
        context.insert("next", &next);
        context.insert("next_size", &next.len());
        context.insert("check", &check);
        context.insert("char_class", char_class.get_raw_ranges());
        context.insert("char_class_size", &char_class.size());
        context.insert("char_class_map", char_class.get_raw_class_map());
        context.insert(
            "ascii_map",
            &char_class
                .get_raw_ascii_mapping()
                .iter()
                .collect::<Vec<_>>(),
        );

        // 渲染模板
        let rendered = tera.render("lex_yy.tera", &context).unwrap();
        fs::write(self.path, rendered).unwrap();
        // println!("{}", rendered);


    }

    /// 压缩DFA矩阵
    fn compress_dfa(&self) -> (Vec<Option<usize>>, Vec<usize>, Vec<Option<StateID>>) {
        let dfa = self.lexer.get_dfa();
        let mut bitmap = bitvec![0; dfa.get_stride()];

        let mut base: Vec<Option<usize>> = vec![None; dfa.size()];
        let mut next: Vec<usize> = vec![0; dfa.get_stride()];
        let mut check: Vec<Option<StateID>> = vec![None; dfa.get_stride()];

        let mut states: Vec<_> = (0..dfa.size()).map(|state| (state, dfa.get_symbols(state))).collect();
        states.sort_by_key(|(_, edges)| edges.len());
        states.reverse();


        for (state, edges) in states {
            if edges.is_empty() {
                continue;
            }

            // 最大转移，内存分配上界
            let max_edge = edges.iter().max().copied().unwrap();

            let offset = Self::alloc_base(&mut bitmap, &edges, max_edge);

            let max_size = offset + max_edge + 1;

            if max_size > next.len() {
                next.resize(max_size, 0);
                check.resize(max_size, None);
            }

            for edge in edges {
                let pos = offset + edge;
                next[pos] = dfa.find_next(state, edge).unwrap();
                check[pos] = Some(state);
            }
            base[state] = Some(offset);
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

        while !Self::try_allocate(bitmap, edges, offset, max_edge) {
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

}

#[test]
fn test_lex_writer() {
    // 构造二维数组
    let table = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
    let mut tera = Tera::default();
    tera.add_raw_template("lex_yy.tera", TEMPLATE).unwrap();
}
