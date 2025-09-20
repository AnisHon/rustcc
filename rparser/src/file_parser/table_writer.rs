//! date: 2025/8/26
//! author: anishan
//!
//! 将表格输出到rust源文件
//!
//!

use crate::common::lr_type::LRAction;
use crate::file_parser::table_builder::LRTableBuilder;
use askama::Template;
use common::utils::compress::compress_matrix;
use common::utils::str_util::{default_cvt, option_cvt, str_option_cvt, string_cvt, vec_to_code};
use heck::ToSnakeCase;
use regex::Regex;
use std::fs;
use crate::file_parser::config_reader::END_SYMBOL_ID;

const ARG_BASE: &str = "_arg";
const VALUE_NAME: &str = "value";

#[derive(Template)]
#[template(path = "parser.rs.askama", ext = "txt", escape = "none")]
pub struct ParserTemplate<'a> {
    decl_code: &'a str,
    user_code: &'a str,
    value_name: &'static str,
    typename: String,
    init_state: usize,
    end_symbol: usize,
    action_base: String,
    action_base_sz: usize,
    action_next: String,
    action_next_sz: usize,
    action_check: String,
    action_check_sz: usize,
    action_row_id: String,
    action_row_id_sz: usize,
    goto_base: String,
    goto_base_sz: usize,
    goto_next: String,
    goto_next_sz: usize,
    goto_check: String,
    goto_check_sz: usize,
    goto_row_id: String,
    goto_row_id_sz: usize,
    expr_lens: String,
    expr_lens_sz: usize,
    expr_names: String,
    expr_names_sz: usize,
    expr_ids: String,
    expr_ids_sz: usize,
    token_contents: String,
    token_contents_sz: usize,
    action_codes: Vec<Option<String>>,
    arguments: Vec<String>,
   
}

#[derive(Template)]
#[template(path = "lex_decl.rs.askama", ext = "txt", escape = "none")]
pub struct LexerDeclTemplate {
    decls: Vec<(String, usize)>
}


/// 生成处理程序
pub struct TableWriter {
    path: String, // 输出路径
    decl_path: String, // 定义输出路径
    builder: LRTableBuilder,
    re_dollar_dollar: Regex,
    re_dollar_num: Regex,
}

impl TableWriter {
    pub fn new(output: &str, decl_path: &str, lr_table_builder: LRTableBuilder) -> Self {
        let re_dollar_dollar = Regex::new(r"\$\$").unwrap();
        let re_dollar_num = Regex::new(r"\$(\d+)").unwrap();
        Self {
            path: output.to_owned(),
            decl_path: decl_path.to_owned(),
            builder: lr_table_builder,
            re_dollar_num,
            re_dollar_dollar
        }
    }


    /// 转换
    fn resolve_action(&self) -> Vec<Option<String>> {
        self.builder.prod_map.iter().map(|prod_meta|
            prod_meta.action.as_ref().map(|x| self.convert_format_regex(x))
        ).collect()
    }

    /// 解析属性文法，替换 $$
    fn convert_format_regex(&self, input: &str) -> String {
        let input = &input[1..input.len() - 1]; // 去除括号{}
        // 先替换 $$
        let step1 = self.re_dollar_dollar.replace_all(input, VALUE_NAME);


        // 再替换 $数字，使用闭包计算新索引
        let result = self.re_dollar_num.replace_all(&step1, |caps: &regex::Captures| {
            // 提取匹配到的数字
            let num_str = &caps[1];
            match num_str.parse::<usize>() {
                Ok(num) => format!("_arg{}.into()", num),
                Err(err) => panic!("{}", err) // 解析失败，
            }
        });

        result.to_string()
    }

    /// 写入文件
    pub fn write(self) {
        self.write_parser();
        self.write_lexer();
    }

    /// 生成parser主体代码
    pub fn write_parser(&self) {
        let (action_table, goto_table, init_state) = self.builder.build_lr_table();

        let decl_code = self.builder.config.decl_code.as_str();

        let typename = self.builder.config.typename.clone();

        let end_symbol = END_SYMBOL_ID;

        // 表达式长度
        let expr_lens: Vec<_> = self.builder.prod_map.iter().map(|meta| meta.len).collect();
        // 解构的变量名字
        let arguments: Vec<_> = expr_lens.iter().map(|&len| {
            let args: Vec<_> = (1usize..=len).map(|x| format!("{}{}", ARG_BASE, x)).collect();
            args.join(", ").to_string()
        }).collect();
        let (expr_lens, expr_lens_sz) = vec_to_code(expr_lens.into_iter(), default_cvt);

        // 表达式名字
        let expr_names: Vec<String> = self.builder.prod_map.iter().map(|x| x.name.clone()).collect();
        let (expr_names, expr_names_sz) = vec_to_code(expr_names.into_iter(), string_cvt);

        // 每个推导式对应的规则ID
        let expr_ids: Vec<_> = self.builder.prod_map.iter().map(|x| x.id).collect();
        let (expr_ids, expr_ids_sz) = vec_to_code(expr_ids.into_iter(), default_cvt);

        // 符号内容（符号名字）
        let token_contents: Vec<Option<String>> = self.builder.token_meta.iter()
            .map(|x| x.as_ref().map(|meta| meta.content.clone()))
            .collect();
        let (token_contents, token_contents_sz) = vec_to_code(token_contents.into_iter(), str_option_cvt);

        // 属性文法代码（解析后）
        let action_codes = self.resolve_action();

        // 压缩矩阵
        let (action_base, action_next, action_check, action_row_id) = compress_matrix(&action_table);
        let (goto_base, goto_next, goto_check, goto_row_id) = compress_matrix(&goto_table);

        // 处理成代码字符串数组
        let (action_base, action_base_sz) = vec_to_code(action_base.into_iter(), option_cvt);
        let (action_next, action_next_sz) = vec_to_code(action_next.into_iter(), action_to_code);
        let (action_check, action_check_sz) = vec_to_code(action_check.into_iter(), option_cvt);
        let (action_row_id, action_row_id_sz) = vec_to_code(action_row_id.into_iter(), default_cvt);

        let (goto_base, goto_base_sz) = vec_to_code(goto_base.into_iter(), option_cvt);
        let (goto_next, goto_next_sz) = vec_to_code(goto_next.into_iter(), option_cvt);
        let (goto_check, goto_check_sz) = vec_to_code(goto_check.into_iter(), option_cvt);
        let (goto_row_id, goto_row_id_sz) = vec_to_code(goto_row_id.into_iter(), default_cvt);


        // 用户代码
        let user_code = self.builder.config.user_code.as_str();


        let template = ParserTemplate {
            decl_code,
            init_state,
            end_symbol,
            typename,
            value_name: VALUE_NAME,
            action_base,
            action_base_sz,
            action_next,
            action_next_sz,
            action_check,
            action_check_sz,
            action_row_id,
            action_row_id_sz,
            goto_base,
            goto_base_sz,
            goto_next,
            goto_next_sz,
            goto_check,
            goto_check_sz,
            goto_row_id,
            goto_row_id_sz,
            expr_lens,
            expr_lens_sz,
            expr_names,
            expr_names_sz,
            expr_ids,
            expr_ids_sz,
            token_contents,
            token_contents_sz,
            action_codes,
            arguments,
            user_code,
        };


        // 渲染模板
        let rendered = template.render().unwrap();
        fs::write(self.path.clone(), rendered).unwrap();
        // println!("{}", rendered);
    }

    /// 生成lexer symbol代码定义
    pub fn write_lexer(&self) {
        let decls: Vec<_> = self.builder.token_meta.iter()
            .flatten() // 过滤None，取出Some
            .filter(|x| !x.is_single) // 过滤单字符
            .map(|meta| {
                let id = meta.id;
                let content = meta.content.to_snake_case().to_uppercase();
                (content, id)
            })
            .collect();

        let template = LexerDeclTemplate { decls };
        let rendered = template.render().unwrap();
        // fs::write(self.decl_path.clone(), rendered).unwrap();
        // println!("{}", rendered);
    }

}
fn action_to_code(action: LRAction) -> String {
    match action {
        LRAction::Reduce(x) => format!("Reduce({})", x),
        LRAction::Shift(x) => format!("Shift({})", x),
        LRAction::Accept(x) => format!("Accept({})", x),
        LRAction::Error => "Error".to_string()
    }
}




