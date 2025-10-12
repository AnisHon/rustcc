use crate::common::grammar::EpsilonSymbol;
use crate::file_parser::table_builder::LRTableBuilder;
use crate::util::first_set::build_first;
use askama::Template;
use std::collections::HashMap;

#[derive(Template)]
#[template(path = "lex_decl.rs.askama", ext = "txt", escape = "none")]
pub struct LexerDeclTemplate {
    decls: Vec<(String, usize)>
}


/// 生成处理程序
pub struct FirstSetWriter {
    builder: LRTableBuilder,
}

impl FirstSetWriter {
    pub fn new(lr_table_builder: LRTableBuilder) -> Self {
        Self {
            builder: lr_table_builder,
        }
    }


    /// 生成parser主体代码
    pub fn write(&self) {
        let grammar = &self.builder.grammar;
        let prods = &self.builder.prod_map;
        let tokens = &self.builder.token_meta;
        let first_set = build_first(grammar);

        let map: HashMap<_, _> = prods.iter()
            .map(|x| (x.id, x.name.clone()))
            .collect();


        for (rule, symbol) in first_set {
            let rule = &map[&rule];
            let tokens: Vec<_> = symbol.into_iter().map(|x| match x {
                EpsilonSymbol::Epsilon => "_".to_owned(),
                EpsilonSymbol::Symbol(x) => format!("{}", tokens[x].as_ref().unwrap().content)
            }).collect();

            let first = tokens.join(", ");

            println!("{} -> {{{}}}", rule, first);
        }
    }
}



