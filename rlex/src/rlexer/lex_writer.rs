use crate::rlexer::lexer::Lexer;
use common::lex::StateID;
use common::utils::compress::compress_matrix;
use heck::ToUpperCamelCase;
use std::fmt::Display;
use std::fs;
use tera::{Context, Tera};

const TEMPLATE: &str = include_str!("../../resources/lex.rs.tera");
pub struct LexWriter {
    path: String,
    lexer: Lexer,
}

impl LexWriter {
    pub fn new(path: &str, lexer: Lexer) -> Self {
        Self { path: path.to_string(), lexer }
    }

    /// Tera真不好用，远不如Thymeleaf JSP
    /// 自己转字符串
    fn optional_to_string<T>(vec: Vec<Option<T>>) -> String
    where T: Display
    {
        let mut value = "[".to_string();
        for x in vec {
            let to_string = match x {
                None => "None",
                Some(x) => &format!("Some({})", x)
            };
            value.push_str(to_string);
            value.push(',')
        }
        value.push(']');
        value
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

        let (base, next, check, row_id) = compress_matrix(self.lexer.get_dfa().get_raw_matrix(), None);
        let next: Vec<StateID> = next.into_iter().map(|x| x.unwrap_or(0)).collect();
        // let (base, next, check) = self.compress_dfa();
        let base = Self::optional_to_string(base);
        let check = Self::optional_to_string(check);

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
        context.insert("row_id", &row_id);
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

}

