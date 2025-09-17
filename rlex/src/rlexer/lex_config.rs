use crate::utils::regex_util::escape_regex_meta;
use pest::iterators::Pairs;
use pest::Parser;
use pest_derive::Parser;

// 词法分析器 结构
#[derive(Debug)]
pub struct LexStruct {
    pub name: String,
    pub regex: String,
    pub skip: bool,
}

///
/// # Members
/// - `decl_code`: 头部声明代码
/// - `decls`: 声明配置，目前没什么用
/// - `rules`: lex规则
/// - `user_code`: 用户代码
#[derive(Debug)]
pub struct LexConfig {
    decl_code: Option<String>,
    decls: Vec<LexDecl>,
    rules: Vec<LexRule>,
    user_code: Option<String>,
}

/// 目前只有这一个配置
#[derive(Debug)]
pub enum LexDecl {
    Option(String)
}

#[derive(Debug)]
pub struct LexRule {
    regex: String,
    action: Option<String>,
}

#[derive(Parser)]
#[grammar = "lexer.pest"]
pub struct LexGrammar;

pub struct LexConfigParser {
    input: String,
}

pub enum PatternType {
    Unquoted,
    Quoted,
}


pub struct LexConfigRule {
    pattern: String,
    pattern_type: PatternType,
    action: Option<String>,
}


impl LexConfigParser {
    pub fn new(input: String) -> LexConfigParser {
        LexConfigParser { input }
    }

    pub fn parse(self) -> LexConfig {


        let file = LexGrammar::parse(Rule::file, self.input.as_str())
            .unwrap()
            .next()
            .unwrap();

        let mut lex_config = LexConfig {
            decl_code: None,
            decls: Vec::new(),
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
                Rule::decls => { /* 目前无用忽略 */ },
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



        println!("{:#?}", lex_config);
        lex_config
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
                    let regex = pattern.as_str();

                    let regex = match pattern.as_rule() {
                        Rule::quoted_pattern => {
                            let regex: &str = &regex[1..regex.len() - 1]; // 去掉引号
                            escape_regex_meta(regex)
                        },
                        Rule::unquoted_pattern => regex.to_string(),
                        _ => unreachable!()
                    };
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

}


#[test]
pub fn t() {
    let input = r#"
%{
#include <stdio.h>
%}

%option noyywrap

%%
[a-z]+    { return IDENT; }
"+"       { return PLUS; }
[0-9]+    { return NUMBER; }
%%

int main() { yylex(); }
"#;


    LexConfigParser::new(input.to_string()).parse();
}
