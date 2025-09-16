use pest::iterators::{Pair, Pairs};
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
/// - `decls`: 声明配置
/// - `rules`: lex规则
/// - `user_code`: 用户代码
#[derive(Debug)]
pub struct LexConfig {
    decl_code: String,
    decls: Vec<LexDecl>,
    rules: Vec<LexRule>,
    user_code: String,
}

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
    decl_code: String,
    decls: Vec<LexDecl>,
    rules: Vec<LexConfigRule>,
    user_code: String,
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
        LexConfigParser {
            input,
            decl_code: String::new(),
            decls: Vec::new(),
            rules: Vec::new(),
            user_code: String::new()
        }
    }

    pub fn parse(mut self) {

        let input = self.input.clone();

        let file = LexGrammar::parse(Rule::file, input.as_str())
            .unwrap()
            .next()
            .unwrap();

        for x in file.into_inner() {
            match x.as_rule() {
                Rule::decl_code => {
                    let user_code: Vec<char> = x.as_str().chars().collect();
                    let user_code: String = user_code[2..user_code.len() - 2].iter().collect();
                    println!("{}", user_code)
                },
                Rule::decls => { /* 目前无用忽略 */ },
                Rule::rules => self.parse_rules(x.into_inner()),
                Rule::user_code => println!("{}", x.as_str()),
                Rule::EOI => {}
                _ => unreachable!()
            }
        }
        // println!("{:?}", file);
    }

    pub fn parse_rules(&mut self, rules: Pairs<Rule>) {
        for rule_decl in rules {
            assert_eq!(rule_decl.as_rule(), Rule::rule_decl);

            let mut regex: Option<String> = None;
            let mut regex_acton: Option<String> = None;

            for pair in rule_decl.into_inner() {
                match pair.as_rule() {
                    Rule::pattern => regex = Some(),
                    Rule::action => regex_acton = Some(pair.as_str().to_string()),
                }


            }

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
