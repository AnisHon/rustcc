//! date: 2025/8/26
//! author: anishan
//!
//! 文法文件读解析
//!
//! # Contents
//! GrammarConfig 解析后的文法字符串结构体
//! AssocType 解析后的符号结合表
//!
//!
//!

use crate::common::grammar::{Assoc, Grammar, ProdMeta, RuleVec, Symbol, SymbolMeta, SymbolVec};
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;
use std::io::BufRead;
use indexmap::IndexMap;
use unescape::unescape;
use common::utils::id_util::IncIDFactory;
use common::utils::unique_id_factory::UniqueIDFactory;

#[derive(Parser)]
#[grammar = "parser_pest.pest"]
struct BisonParser;

/// 文法结构，相当于文法配置的AST
///
/// # Members
/// - 'tokens': token声明
/// - 'assoc': 结核性声明
/// - 'productions': 文法推导式
/// - 'user_code': 用户代码区域
///
#[derive(Debug)]
pub struct GrammarConfig {
    pub tokens: Vec<String>,
    pub assoc: Vec<AssocType>,
    pub productions: Vec<Production>,
    pub user_code: String,
    pub typename: String,
}

/// Token结核性表类型
#[derive(Debug)]
pub enum AssocType {
    Left(Vec<String>),
    Right(Vec<String>),
    NonAssoc(Vec<String>),
}

impl AssocType {
    pub fn unwrap(&self) -> &Vec<String> {
        match self {
            AssocType::Left(x) => x,
            AssocType::Right(x) => x,
            AssocType::NonAssoc(x) => x
        }
    }
}


/// 文法推导式结构
///
/// # Members
/// - 'name': 文法名
/// - 'rules': 文法相关表格
///     - 'Vec<String>': 文法符号数组
///     - 'Option<String>': 属性文法代码
///     - 'Assoc': 文法结合性，默认为None
#[derive(Debug)]
pub struct Production {
    pub name: String,
    pub rules: Vec<(Vec<String>, Option<String>, Assoc)>, // 推导式 action代码 结核性
}


/// 文法配置构造器
/// 
/// # Members
/// - 'input': 输入文法
/// - 'tokens': token声明
/// - 'assoc': 结合性声明
/// - 'productions': 文法推导式声明
/// - 'user_code': 用户代码
/// 
pub struct GrammarConfigParser {
    input: String,
    tokens: Vec<String>,
    assoc: Vec<AssocType>,
    productions: Vec<Production>,
    user_code: String,
    typename: String
}

impl GrammarConfigParser {
    pub fn new(input: String) -> Self {
        Self { input, tokens: Vec::new(), assoc: Vec::new(), productions: Vec::new(), user_code: String::new(), typename: String::new() }
    }

    /// 解析文法
    pub fn parse(mut self) -> GrammarConfig {
        let input = self.input.clone();
        let pairs = BisonParser::parse(Rule::file, input.as_str()).unwrap();



        for pair in pairs {
            self.parse_file(pair);
        }
        GrammarConfig { tokens: self.tokens, assoc: self.assoc, productions: self.productions, user_code: self.user_code, typename: self.typename }
    }

    fn parse_file(&mut self, file: Pair<Rule>) {
        for rule in file.into_inner() {
            match rule.as_rule() {
                Rule::decls => self.parse_decls(rule),
                Rule::rules => self.parse_rules(rule),
                Rule::user_code => self.parse_user_code(rule),
                Rule::EOI => (),
                _ => unreachable!()
            }
        }
    }
    /// 解析 decl
    fn parse_decls(&mut self, decls: Pair<Rule>) {
        for decl in decls.into_inner() {
            for pair in decl.into_inner() {
                match pair.as_rule() {
                    Rule::token_decl => self.parse_token_decl(pair),
                    Rule::assoc_decl => self.parse_assoc_decl(pair),
                    Rule::type_decl => self.parse_type_decl(pair),
                    _ => unreachable!(),
                }
            }
        }

    }

    /// token声明
    fn parse_token_decl(&mut self, decl: Pair<Rule>) {
        self.tokens.extend(decl.into_inner().into_iter().map(|x| x.as_str().to_string()));
    }
    
    /// 结合性声明
    fn parse_assoc_decl(&mut self, decl: Pair<Rule>) {
        let assoc_type = &decl.as_span().as_str().split_whitespace().next().unwrap()[1..];
        let assoc_values: Vec<String> = decl.into_inner().map(|x| x.as_span().as_str().to_string()).collect();
        let assoc = match assoc_type {
            "left" => AssocType::Left(assoc_values),
            "right" => AssocType::Right(assoc_values),
            "nonassoc" => AssocType::NonAssoc(assoc_values),
            _ => unreachable!()
        };
        self.assoc.push(assoc);
    }

    /// type声明
    fn parse_type_decl(&mut self, decl: Pair<Rule>) {
        self.typename = decl.into_inner().as_str().to_string();
    }

    /// 解析文法相关
    fn parse_rules(&mut self, rules: Pair<Rule>) {
        for rule_decl in rules.into_inner() {
            let production = Self::parse_rule_decl(rule_decl);
            self.productions.push(production);
        }
    }

    /// 单个推导式声明
    fn parse_rule_decl(rule_decl: Pair<Rule>) -> Production {
        let mut pairs = rule_decl.into_inner();
        // pairs.for_each(|pair| println!("{:?}", pair));
        let mut production = Production {
            name: pairs.next().unwrap().as_str().to_string(),
            rules: Vec::new(),
        };
    
        // pairs 内层是 prec_production
        for pair in pairs.into_iter() {
            let mut symbols = Vec::new();
            let mut action = None;
            let mut prec_directive = Assoc::None;

            // prec_production 内层是 production和prec_type(%prec)
            for prec_prod in pair.into_inner() {

                // 解析prec_type
                if matches!(prec_prod.as_rule(), Rule::prec_directive) { // 处理优先级
                    let mut prec_pairs = prec_prod.into_inner();
                    let prec_type = prec_pairs.next().unwrap().as_str();
                    prec_directive = match prec_type {
                        "left" => Assoc::Left,
                        "right" => Assoc::Right,
                        "nonassoc" => Assoc::NonAssoc,
                        _ => unreachable!()
                    };
                    continue
                }

                //  production 内层是 symbol和action
                for item in prec_prod.into_inner() {
                    match item.as_rule() {
                        Rule::symbol => symbols.push(item.as_str().to_string()),
                        Rule::action => action = Some(item.as_str().to_string()),
                        _ => unreachable!()
                    }
                }
            }


            production.rules.push((symbols, action, prec_directive));
        }


        production
    }
    
    fn parse_user_code(&mut self, rules: Pair<Rule>) {
        self.user_code = rules.as_str().to_string();
    }

}


///
/// 将GrammarConfig转换成 Grammar SymbolMeta表 和 ProdMeta表
///
/// # Arguments
/// - 'config': GrammarConfig
/// - 'lex_tokens': lex的Token，用于对齐lex生成的ID
///
/// # Returns
/// - 'Grammar<usize>': grammar对象
/// - 'Vec<SymbolMeta>': symbol_id -> Symbol Meta
/// - 'Vec<ProdMeta>': production_id -> Production Meta
///
pub fn get_grammar(config: &GrammarConfig) -> (Grammar<usize>, Vec<SymbolMeta>, Vec<ProdMeta>) {

    let mut grammar = Grammar::new(0);

    let non_terminal: HashMap<String, usize> = config.productions.iter().enumerate()// 非终结符Name -> ID
        .map(|(idx, production)| (production.name.clone(), idx))
        .collect();

    let (token_meta, token_map) = build_token_map(config);

    // production的meta信息
    let mut prod_map = Vec::new();

    // 构建推导式
    for (rule_id, production) in config.productions.iter().enumerate() {
        let mut rule_vec = RuleVec::new();

        // 遍历所有alter
        for (alter, (symbols, action, assoc)) in production.rules.iter().enumerate() {
            let mut prod_meta = ProdMeta::new(rule_id, alter, symbols.len(), production.name.clone()); // 构建Meta

            prod_meta.action = action.clone();

            let rule = if symbols.is_empty() { // 空 epsilon
                crate::common::grammar::Rule::Epsilon
            } else {
                // 非空，遍历所有symbol
                let symbol_vec: SymbolVec<_> = symbols.iter().map(|symbol|{
                    if non_terminal.contains_key(symbol) { // 非终结符
                        Symbol::NonTerminal(non_terminal[symbol]) // 查Rule ID
                    } else { // 终结符
                        let tid = *token_map.get(symbol).unwrap_or_else(|| panic!("No Such Token {}", symbol));
                        let meta = &token_meta[tid];
                        prod_meta.priority = meta.priority;   // 推导式以最后的终结符为准
                        prod_meta.assoc = meta.assoc; // 最后终结符的assoc
                        Symbol::Terminal(tid)
                    }
                }).collect();
                crate::common::grammar::Rule::Expression(symbol_vec)
            };

            // 选择覆盖还是使用终结符assoc
            prod_meta.assoc = match assoc {
                Assoc::None => prod_meta.assoc, // 未确定
                _ => *assoc  // 已确定覆盖
            };

            prod_map.push(prod_meta);
            rule_vec.push(rule);
        }

        grammar.add_rule(rule_id, rule_vec);
    }

    (grammar, token_meta, prod_map)
}

fn build_token_map(config: &GrammarConfig) -> (Vec<SymbolMeta>, HashMap<String, usize>) {
    let mut id_factory = IncIDFactory::new(258); // 从258之后开始编码，256~257保留
    let mut token_meta: Vec<Option<SymbolMeta>> = Vec::new();

    let mut token_map: IndexMap<String, usize> = IndexMap::new();

    for (idx, assoc) in config.assoc.iter().enumerate() {
        let assoc_type = match assoc {
            AssocType::Right(_) => Assoc::Right,
            AssocType::Left(_) => Assoc::Left,
            AssocType::NonAssoc(_) => Assoc::NonAssoc,
        };
        assoc.unwrap().iter().for_each(|x| {
            let token_id = *token_map
                .entry(x.clone()).or_insert_with(|| get_symbol_id(x, &mut id_factory));
            let mut meta = SymbolMeta::new(token_id, x.clone());
            meta.priority = idx;
            meta.assoc = assoc_type;
            token_meta[token_id] = Some(meta);
        });
    }
    (token_meta, token_map)
}

fn is_single_symbol(symbol: &str) -> bool {
    symbol.starts_with("'") && symbol.ends_with("'")
}

/// 获取symbol的id，如果是但字符串则返回ASCII码，否则生成ID
fn get_symbol_id(symbol: &str, id_factory: &mut IncIDFactory) -> usize {
    let single = symbol.starts_with("'") && symbol.ends_with("'");

    if single {
        let symbol = unescape(&symbol[1usize..symbol.len() - 1]).unwrap();
        assert_eq!(symbol.len(), 1, "unsupported symbol '{}'", symbol);
        symbol.chars().next().unwrap() as usize
    } else {
        id_factory.next_id()
    }
}

/// 读取lexer文件中的token，相互对应
pub fn token_from_lexer(buffer: impl BufRead) -> Vec<String> {
    let mut lex: Vec<String> = Vec::new();

    for line in buffer.lines() {
        let line = line.unwrap();
        let vec: Vec<_> = line.split_whitespace().collect();

        if vec.is_empty() {
            continue;
        }
        let name: String = vec[0].to_string();

        lex.push(name);
    }

    lex
}