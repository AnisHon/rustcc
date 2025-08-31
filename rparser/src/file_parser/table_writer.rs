//! date: 2025/8/26
//! author: anishan
//!
//! 将表格输出到rust源文件
//!
//!

use crate::common::lr_type::LRAction;
use crate::file_parser::table_builder::LRTableBuilder;
use common::utils::compress::compress_matrix;
use common::utils::str_util::option_to_code_str;
use regex::Regex;
use std::fs;
use tera::{Context, Tera};

const TEMPLATE: &str = include_str!("../../resources/parser.rs.tera");
const VALUE_STACK_NAME: &str = "value_stack";
const VALUE_NAME: &str = "value";

/// 生成处理程序
pub struct TableWriter {
    path: String, // 输出路径
    builder: LRTableBuilder,
    re_dollar_dollar: Regex,
    re_dollar_num: Regex,
}

impl TableWriter {
    pub fn new(output: &str, lr_table_builder: LRTableBuilder) -> Self {
        let re_dollar_dollar = Regex::new(r"\$\$").unwrap();
        let re_dollar_num = Regex::new(r"\$(\d+)").unwrap();
        Self {
            path: output.to_string(),
            builder: lr_table_builder,
            re_dollar_num,
            re_dollar_dollar
        }
    }


    /// 转换
    fn resolve_action(&self) -> Vec<Option<String>> {
        self.builder.prod_map.iter().map(|prod_meta|  {
            let action = prod_meta.action.as_ref();
            match action {
                None => return None,
                Some(x) => Some(self.convert_format_regex(x))
            }
        }).collect()
    }

    /// 解析属性文法，替换 $$
    fn convert_format_regex(&self, input: &str) -> String {
        // 先替换 $$
        let step1 = self.re_dollar_dollar.replace_all(input, VALUE_NAME);


        // 再替换 $数字，使用闭包计算新索引
        let result = self.re_dollar_num.replace_all(&step1, |caps: &regex::Captures| {
            // 提取匹配到的数字
            let num_str = &caps[1];
            match num_str.parse::<usize>() {
                Ok(num) => format!("mem::take(&mut {}[{}])", VALUE_STACK_NAME, num - 1),  // 数字减1得到新索引
                Err(err) => panic!("{}", err) // 解析失败，
            }
        });

        result.to_string()
    }

    /// 写入文件
    pub fn write(self) {
        let (action_table, goto_table, init_state) = self.builder.build_lr_table();

        let typename = self.builder.config.typename.clone();

        // 结束符号在所有token之后
        let end_symbol = self.builder.token_meta.len();
        // 表达式长度
        let expr_lens: Vec<_> = self.builder.prod_map.iter().map(|meta| meta.len).collect();
        // 表达式名字
        let expr_names: Vec<String> = self.builder.prod_map.iter().map(|x| x.name.clone()).collect();
        // 每个推导式对应的规则ID
        let expr_ids: Vec<_> = self.builder.prod_map.iter().map(|x| x.id).collect();

        // 符号内容（符号名字）
        let token_contents: Vec<String> = self.builder.token_meta.iter().map(|x| x.content.clone()).collect();

        // 属性文法代码（解析后）
        let action_codes = self.resolve_action();

        // 压缩矩阵
        let (action_base, action_next, action_check, action_row_id) = compress_matrix(&action_table, LRAction::Error);
        let (goto_base, goto_next, goto_check, goto_row_id) = compress_matrix(&goto_table, None);

        // 处理成代码字符串数组
        let action_base = option_to_code_str(action_base);
        let action_next = action_to_code_str(action_next);
        let action_check = option_to_code_str(action_check);
        let goto_base = option_to_code_str(goto_base);
        let goto_next = option_to_code_str(goto_next);
        let goto_check = option_to_code_str(goto_check);

        // 用户代码
        let user_code = self.builder.config.user_code;

        let mut tera = Tera::default();
        tera.add_raw_template("parser.rs.tera", TEMPLATE).unwrap();
        let mut context = Context::new();

        context.insert("action_base", &action_base);
        context.insert("action_next", &action_next);
        context.insert("action_check", &action_check);
        context.insert("action_row_id", &action_row_id);
        context.insert("goto_base", &goto_base);
        context.insert("goto_next", &goto_next);
        context.insert("goto_check", &goto_check);
        context.insert("goto_row_id", &goto_row_id);
        context.insert("expr_lens", &expr_lens);
        context.insert("expr_names", &expr_names);
        context.insert("expr_ids", &expr_ids);
        context.insert("token_contents", &token_contents);
        context.insert("action_codes", &action_codes);
        context.insert("vs_name", VALUE_STACK_NAME);
        context.insert("cv_name", VALUE_NAME);
        context.insert("user_code", &user_code);
        context.insert("init_state", &init_state);
        context.insert("end_symbol", &end_symbol);
        context.insert("typename", &typename);

        
        

        // 渲染模板
        let rendered = tera.render("parser.rs.tera", &context).unwrap();
        fs::write(self.path, rendered).unwrap();
    }



}
fn action_to_code_str(vec: Vec<LRAction>) -> Vec<String> {
    vec.into_iter().map(|action| match action {
        LRAction::Reduce(x) => format!("Reduce({})", x),
        LRAction::Shift(x) => format!("Shift({})", x),
        LRAction::Accept(x) => format!("Accept({})", x),
        LRAction::Error => "Error".to_string()
    }).collect()
}




