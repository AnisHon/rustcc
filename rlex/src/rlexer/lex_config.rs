use indexmap::IndexMap;
use crate::utils::regex_util::escape_regex_meta;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;

// // 词法分析器 结构
// #[derive(Debug)]
// pub struct LexStruct {
//     pub name: String,
//     pub regex: String,
//     pub skip: bool,
// }

///
/// # Members
/// - `decl_code`: 头部声明代码
/// - `decls`: 声明配置，目前没什么用
/// - `rules`: lex规则
/// - `user_code`: 用户代码
#[derive(Debug)]
pub struct LexConfig {
    pub decl_code: Option<String>,
    pub options: Vec<String>,
    pub params: Vec<String>,
    pub typ: String,
    pub rules: Vec<LexRule>,
    pub user_code: Option<String>,
}

#[derive(Debug)]
pub struct LexRule {
    pub regex: String,
    pub action: Option<String>,
}

#[derive(Parser)]
#[grammar = "lexer.pest"]
pub struct LexGrammar;

pub struct LexConfigParser {
    input: String,
    pattern_decl: IndexMap<String, String>,
}

impl LexConfigParser {
    pub fn new(input: String) -> LexConfigParser {
        LexConfigParser { input, pattern_decl: IndexMap::new() }
    }

    pub fn parse(mut self) -> LexConfig {
        let input = self.input.clone();
        let file = LexGrammar::parse(Rule::file, input.as_str())
            .unwrap()
            .next()
            .unwrap();

        let mut lex_config = LexConfig {
            decl_code: None,
            options: Vec::new(),
            params: Vec::new(),
            typ: "usize".to_string(),
            rules: Vec::new(),
            user_code: None,
        };

        for x in file.into_inner() {
            match x.as_rule() {
                Rule::decl_code => {
                    let code: Vec<char> = x.as_str().chars().collect();
                    let code: String = code[2..code.len() - 2].iter().collect();
                    lex_config.decl_code = Some(code);
                },
                Rule::decls => { self.parse_decls(x, &mut lex_config) },
                Rule::pattern_decls => { // pattern声明
                    self.parse_pattern_decls(x);
                }
                Rule::rules => {
                    lex_config.rules = self.parse_rules(x.into_inner());
                },
                Rule::user_code => {
                    lex_config.user_code = Some(x.as_str().to_string());
                },
                Rule::EOI => {}
                _ => unreachable!()
            }
        }

        lex_config
    }

    pub fn parse_decls(&self, pair: Pair<Rule>, config: &mut LexConfig) {
        for decl in pair.into_inner() {
            match decl.as_rule() {
                Rule::type_decl => {
                    let typ = decl.into_inner().as_str();
                    config.typ = typ.to_string();
                }
                Rule::option_decl => {
                    for ident in decl.into_inner() {
                        config.options.push(ident.as_str().to_string());
                    }
                }
                Rule::param_decl => {
                    let param = decl.into_inner().as_str().trim();
                    config.params.push(param.to_string());
                }
                _ => unreachable!()
            }

        }


    }

    pub fn parse_pattern_decls(&mut self, x: Pair<Rule>) {
        for pattern_decl in x.into_inner() {
            let mut pattern_decl = pattern_decl.into_inner();
            let ident = pattern_decl.next().unwrap().as_str().to_owned();
            let pattern = pattern_decl.next().unwrap().into_inner().next().unwrap();
            let pattern = self.parse_pattern(pattern);

            self.pattern_decl.insert(ident, pattern);
        }
    }

    pub fn parse_rules(&self, rules: Pairs<Rule>) -> Vec<LexRule> {
        let mut lex_rules: Vec<LexRule> = Vec::new();
        for rule_decl in rules {
            assert_eq!(rule_decl.as_rule(), Rule::rule_decl);
            let rule = self.parse_rule(rule_decl.into_inner());
            lex_rules.push(rule);
        }
        lex_rules
    }

    pub fn parse_rule(&self, rule: Pairs<Rule>) -> LexRule {
        let mut rule_regex: Option<String> = None;
        let mut rule_action: Option<String> = None;

        for pair in rule {
            match pair.as_rule() {
                Rule::pattern => {
                    let pattern = pair.into_inner().next().unwrap();
                    let regex = self.parse_pattern(pattern);
                    rule_regex = Some(regex);
                },
                Rule::action => rule_action = Some(pair.as_str().to_string()),
                _ => unreachable!()
            }
        }

        LexRule {
            regex: rule_regex.unwrap(),
            action: rule_action,
        }
    }

    pub fn parse_pattern(&self, pattern: Pair<Rule>) -> String {
        let regex = pattern.as_str();
        match pattern.as_rule() {
            Rule::quoted_pattern => {
                let regex: &str = &regex[1..regex.len() - 1]; // 去掉引号
                escape_regex_meta(regex)
            },
            Rule::unquoted_pattern => {
                // 试图对占位符格式化
                self.format_pattern(regex)
            },
            _ => unreachable!()
        }
    }

    /// 填充模版
    pub fn format_pattern(&self, input: &str) -> String {
        let mut out = String::with_capacity(input.len());
        let mut i = 0usize;
        let input: Vec<_>= input.chars().collect();
        let len = input.len();

        while i < len {
            let ch = input[i];

            // 是否在 '{' 之后，遇到 '}' 停止
            let lbrace = ch == '{';

            // 不在{ 中直接推入
            if !lbrace {
                out.push(ch);
                i += 1;
                continue;
            }

            // 在 { 中判断是否合法
            let lbrace_idx = i;
            let mut rbrace_idx = i + 1;
            let mut valid = false;

            // 检查 } 边界，和是否是有效占位
            while rbrace_idx < len {
                let ch = input[rbrace_idx];
                let legal = ch.is_ascii_alphanumeric() || ch == '_';

                // 如果遇到 '}' 且长度大于1，则是合法的占位
                if ch == '}' && rbrace_idx - lbrace_idx > 1 {
                    valid = true;
                    break;
                }

                // 字符不合法直接结束
                if !legal {
                    break;
                }

                rbrace_idx += 1;
            }

            // 合法则查表
            let content: String = match valid {
                true => {
                    let name: String = input[lbrace_idx + 1..=rbrace_idx - 1].iter().collect();
                    let content = self.pattern_decl.get(name.as_str())
                        .unwrap_or_else(|| panic!("Not found {}", name));
                    // 防止出现优先级问题，加上括号
                    format!("({})", content)
                }
                false => input[lbrace_idx..rbrace_idx].iter().collect()
            };

            // 推入
            out.push_str(&content);

            // 跳到下一个
            i = rbrace_idx + 1;
        }

        out

    }

}