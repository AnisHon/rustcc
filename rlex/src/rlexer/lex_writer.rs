use crate::rlexer::lexer::Lexer;
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
        tera.add_raw_template("lex.rs.tera", TEMPLATE).unwrap();
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

        // 准备上下文
        let mut context = Context::new();
        context.insert("state_map_size", &state_map.len());
        context.insert("state_map", &state_map);
        context.insert("enums", &enum_name);
        context.insert("table", dfa.get_raw_matrix());
        context.insert("rows", &dfa.size());
        context.insert("stride", &dfa.get_stride());
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
        let rendered = tera.render("lex.rs.tera", &context).unwrap();
        fs::write(self.path, rendered).unwrap();
        // println!("{}", rendered);
    }
}

#[test]
fn test_lex_writer() {
    // 构造二维数组
    let table = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
    let mut tera = Tera::default();
    tera.add_raw_template("lex.rs.tera", TEMPLATE).unwrap();
}
