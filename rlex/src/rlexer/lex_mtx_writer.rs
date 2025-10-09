use crate::rlexer::lexer::Lexer;
use askama::Template;
use common::utils::compress::compress_matrix;
use std::fs;
use common::lex::{StateID, StateMeta};
use common::utils::str_util::{default_cvt, option_cvt, vec_to_code};

#[derive(Template)]
#[template(path = "lex_mtx.rs.askama", ext = "txt", escape = "none")]
struct LexTemplate<'a> {
    decl_code: Option<&'a str>,
    actions: Vec<Option<String>>,
    init_state: usize,
    base: String,
    base_sz: usize,
    next: String,
    next_sz: usize,
    check: String,
    check_sz: usize,
    row_id: String,
    row_id_sz: usize,
    typ: String,
}

pub struct LexMtxWriter {
    path: String,
    lexer: Lexer,
}

impl LexMtxWriter {
    pub fn new(path: &str, lexer: Lexer) -> Self {
        Self { path: path.to_string(), lexer }
    }

    pub fn write(self) {
        let dfa = self.lexer.get_dfa();

        let config = self.lexer.get_config();
        
        let decl_code = config.decl_code.as_deref();
        
        let actions: Vec<Option<String>> = dfa.get_states().iter()
            .map(|x| match x {
                None => None,
                Some(x) => x.action.as_ref()
            })
            .map(|action| action.map(|x| x[1..x.len() - 1].trim().to_owned()))// 去除开头结尾的{、}和空格
            .collect();

        let (base, next, check, row_id) =
            compress_matrix(dfa.get_raw_matrix());
        let next = next.iter().map(|x| x.unwrap_or_default());

        // 转换成代码
        let (base, base_sz) = vec_to_code(base.into_iter(), option_cvt);
        let (next, next_sz) = vec_to_code(next.into_iter(), default_cvt);
        let (check, check_sz) = vec_to_code(check.into_iter(), option_cvt);
        let (row_id, row_id_sz) = vec_to_code(row_id.into_iter(), default_cvt);

        let template = LexTemplate {
            decl_code,
            base,
            base_sz,
            next,
            next_sz,
            check,
            check_sz,
            row_id,
            row_id_sz,
            actions,
            typ: config.typ.clone(),
            init_state: dfa.get_init_state(),

        };

        let string = template.render().unwrap();
        // println!("{}", string);
        fs::write(&self.path, string).unwrap();
    }

}

