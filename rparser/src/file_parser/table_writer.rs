use std::fs;
use std::num::ParseIntError;
use tera::{Context, Tera};
use common::utils::compress::compress_matrix;
use regex::Regex;
use crate::common::lr_type::LRAction;
use crate::file_parser::table_builder::{LRTableBuilder, TableType};
const TEMPLATE: &str = include_str!("../../resources/parser.rs.tera");
const VALUE_STACK_NAME: &str = "VALUE_STACK";
const VALUE_NAME: &str = "value";

/// 生成处理程序
pub struct TableWriter {
    path: String, // 输出路径
    lr_table_builder: LRTableBuilder,
    re_dollar_dollar: Regex,
    re_dollar_num: Regex,
}

impl TableWriter {
    pub fn new(output: &str, lr_table_builder: LRTableBuilder) -> Self {
        let re_dollar_dollar = Regex::new(r"\$\$").unwrap();
        let re_dollar_num = Regex::new(r"\$(\d+)").unwrap();
        Self {
            path: output.to_string(),
            lr_table_builder,
            re_dollar_num,
            re_dollar_dollar
        }
    }


    /// 转换
    fn resolve_action(&self) -> Vec<Option<String>> {
        let mut action_map = Vec::new();
        action_map.resize(self.lr_table_builder.rule_map.len(), None);

        for (&idx, &(id, pos)) in self.lr_table_builder.rule_map.iter() {
            let action = &self.lr_table_builder.grammar.get_meta(id).unwrap().action[pos];
            let action = match action {
                None => continue,
                Some(x) => x
            };

            let action = self.convert_format_regex(action);
            action_map[idx] = Some(action);
        };
        action_map
    }

    fn convert_format_regex(&self, input: &str) -> String {
        // 先替换 $$
        let step1 = self.re_dollar_dollar.replace_all(input, VALUE_NAME);

        // 再替换 $数字，使用闭包计算新索引
        let result = self.re_dollar_num.replace_all(&step1, |caps: &regex::Captures| {
            // 提取匹配到的数字
            let num_str = &caps[1];
            match num_str.parse::<usize>() {
                Ok(num) => format!("{}[{}]", VALUE_STACK_NAME, num - 1),  // 数字减1得到新索引
                Err(err) => panic!("{}", err) // 解析失败，
            }
        });

        result.to_string()
    }

    pub fn write(self) {
        let (action_table, goto_table, init) = self.lr_table_builder.build_lr_table();

        let action_map = self.resolve_action();
        // 压缩矩阵
        let (action_base, action_next, action_check) = compress_matrix(&action_table, LRAction::Error);
        let (goto_base, goto_next, goto_check) = compress_matrix(&goto_table, None);

        let mut tera = Tera::default();
        tera.add_raw_template("parser.rs.tera", TEMPLATE).unwrap();
        let mut context = Context::new();

        // 渲染模板
        let rendered = tera.render("lex_yy.tera", &context).unwrap();
        fs::write(self.path, rendered).unwrap();


    }



}





