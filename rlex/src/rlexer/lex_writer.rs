use crate::rlexer::lexer::Lexer;
use askama::Template;
use common::utils::compress::compress_matrix;
use std::fs;
use common::utils::str_util::{default_cvt, option_cvt, vec_to_code};

#[derive(Template)]
#[template(path = "lex.rs.askama", ext = "txt", escape = "none")]
struct LexTemplate<'a> {
    decl_code: Option<&'a str>,
    user_code: Option<&'a str>,
    actions: Vec<(usize, &'a str)>,
    params: Vec<String>,
    typ: String,
    init_state: usize,
    base: String,
    base_sz: usize,
    next: String,
    next_sz: usize,
    check: String,
    check_sz: usize,
    row_id: String,
    row_id_sz: usize,
    ranges: String,
    ranges_sz: usize,
    class_map: String,
    class_map_sz: usize,
    ascii_map: String,
    ascii_map_sz: usize
}

pub struct LexWriter {
    path: String,
    lexer: Lexer,
}

impl LexWriter {
    pub fn new(path: &str, lexer: Lexer) -> Self {
        Self { path: path.to_string(), lexer }
    }

    pub fn write(self) {
        let dfa = self.lexer.get_dfa();

        // 将所有终结符打包成
        let actions: Vec<(usize, &str)> = dfa.get_states().iter()
            .enumerate() // enumerate就是ID
            .filter_map(|(state, meta)| Some((state, meta.as_ref()?))) // 过滤不存在的，并解包
            .filter_map(|(state, meta)| Some((state, meta.action.as_ref()?))) // 过滤没有action的，包括非终结状态和部分终结状态
            .map(|(state, meta)| (state, meta.as_str()))
            .collect();

        // println!("{:#?}", dfa.get_states());

        let config = self.lexer.get_config();

        let decl_code = config.decl_code.as_deref();
        let user_code = config.user_code.as_deref();

        let char_class = self.lexer.get_char_class_set();

        let ranges = char_class.get_raw_ranges();
        let class_map = char_class.get_raw_class_map();
        let ascii_map = char_class.get_raw_ascii_mapping();

        let (ranges, ranges_sz) = vec_to_code(
            ranges.into_iter(),
            |(l, r)| format!("({}, {})", l, r)
        );
        let (class_map, class_map_sz) = vec_to_code(class_map.into_iter(), default_cvt);
        let (ascii_map, ascii_map_sz) = vec_to_code(ascii_map.into_iter(), default_cvt);


        // 压缩矩阵
        let (base, next, check, row_id) =
            compress_matrix(dfa.get_raw_matrix());
        // next没有Option语义
        let next = next.iter().map(|x| x.unwrap_or_default());


        // 转换成代码
        let (base, base_sz) = vec_to_code(base.into_iter(), option_cvt);
        let (next, next_sz) = vec_to_code(next.into_iter(), default_cvt);
        let (check, check_sz) = vec_to_code(check.into_iter(), option_cvt);
        let (row_id, row_id_sz) = vec_to_code(row_id.into_iter(), default_cvt);

        let template = LexTemplate {
            decl_code,
            user_code,
            actions,
            params: config.params.clone(),
            typ: config.typ.clone(),
            init_state: dfa.get_init_state(),
            base,
            base_sz,
            next,
            next_sz,
            check,
            check_sz,
            row_id,
            row_id_sz,
            ranges,
            ranges_sz,
            class_map,
            class_map_sz,
            ascii_map,
            ascii_map_sz,
        };

        let rendered = template.render().unwrap();

        fs::write(self.path, rendered).unwrap();
        // println!("{}", rendered);
    }

}

