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
use common::utils::id_util::IncIDFactory;
use common::utils::unique_id_factory::UniqueIDFactory;
use indexmap::IndexSet;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;
use unescape::unescape;

/// 从258之后开始编码，256~257保留
pub const SYMBOL_ID_BEGIN: usize = 258;

/// EOF结束符号ID
pub const END_SYMBOL_ID: usize = 0;

#[derive(Parser)]
#[grammar = "parser_pest.pest"]
struct BisonParser;

/// 文法结构，相当于文法配置的AST
///
/// # Members
/// - `decl_code`: 源码导入声明语句
/// - `tokens`: %token声明
/// - `assoc`: 结核性声明
/// - `typename`: 类型信息
/// - `productions`: 文法推导式
/// - `user_code`: 用户代码区域
///
#[derive(Debug)]
pub struct GrammarConfig {
    pub decl_code: String,
    pub tokens: Vec<String>,
    pub assoc: Vec<AssocType>,
    pub params: Vec<String>,
    pub typename: String,
    pub productions: Vec<Production>,
    pub user_code: String,
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
    pub rules: Vec<(Vec<ConfigSymbol>, Option<String>, Assoc)>, // 推导式 action代码 推导式的结核性
}

#[derive(Debug)]
pub struct ConfigSymbol {
    pub content: String,
    pub kind: SymbolKind
}

impl ConfigSymbol {
    pub fn is_single(&self) -> bool {
        match self.kind {
            SymbolKind::Single => true,
            SymbolKind::Multi => false,
        }
    }
}

#[derive(Debug)]
pub enum SymbolKind {
    Single, // 无需声明
    Multi, // 需要声明
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
}

impl GrammarConfigParser {
    pub fn new(input: String) -> Self {
        Self { input }
    }

    /// 解析文法
    pub fn parse(mut self) -> GrammarConfig {
        let input = self.input.clone();
        let pairs = BisonParser::parse(Rule::file, input.as_str()).unwrap();


        // 把file展开称内部元素
        let pairs = pairs.into_iter()
            .flat_map(|x| x.into_inner());

        let mut config = GrammarConfig {
            params: Vec::new(),
            decl_code: String::new(),
            typename: String::new(),
            tokens: Vec::new(),
            assoc: Vec::new(),
            productions: Vec::new(),
            user_code: String::new(),
        };

        // 遍历内部元素
        for pair in pairs {
            match pair.as_rule() {
                Rule::decl_code => {
                    let decl_code = pair.as_str();
                    let decl_code = decl_code[2..decl_code.len() - 2].trim().to_owned();
                    config.decl_code = decl_code;
                }
                Rule::decls => {
                    Self::parse_decls(pair, &mut config);
                },
                Rule::rules => {
                    let productions = Self::parse_rules(pair);
                    config.productions = productions;
                },
                Rule::user_code => {
                    let user_code = self.parse_user_code(pair);
                    config.user_code = user_code;
                },
                Rule::EOI => {}
                _ => unreachable!(),
            }
        }
        config
    }

    fn parse_symbol(pair: Pair<Rule>) -> ConfigSymbol {
        let symbol = pair.into_inner().next().unwrap();
        let content = symbol.as_str().to_owned();
        let kind = match symbol.as_rule() {
            Rule::quoted_symbol => SymbolKind::Single,
            Rule::unquoted_symbol => SymbolKind::Multi,
            _ => unreachable!()
        };
        ConfigSymbol {
            kind,
            content,
        }
    }

    /// 解析 decl
    fn parse_decls(decls: Pair<Rule>, config: &mut GrammarConfig) {

        for decl in decls.into_inner() {
            let decl = decl.into_inner().next().unwrap();

            match decl.as_rule() {
                Rule::token_decl => {
                    config.tokens.extend(Self::parse_token_decl(decl))
                },
                Rule::assoc_decl => {
                    config.assoc.push(Self::parse_assoc_decl(decl))
                },
                Rule::type_decl => {
                    let ident = decl.into_inner().next().unwrap();
                    config.typename.push_str(ident.as_str());
                },
                Rule::param_decl => {
                    let param = decl.into_inner().next().unwrap().as_str().to_owned();
                    config.params.push(param);
                }
                _ => unreachable!(),
            };
        }
        
    }

    /// %token ...
    /// token_decl = { "%token" ~ ident+ }
    ///
    fn parse_token_decl(pair: Pair<Rule>) -> Vec<String> {
        pair.into_inner()
            .map(|x| x.as_str().to_owned())
            .collect()
    }

    ///
    /// %left/right/nonassoc
    /// assoc_decl = { ("%" ~ assoc_type) ~ symbol+ }
    fn parse_assoc_decl(pair: Pair<Rule>) -> AssocType {
        let mut pairs = pair.into_inner();

        let assoc_type = pairs.next().unwrap().as_str();

        let symbols: Vec<_> = pairs
            .map(|x| Self::parse_symbol(x).content)
            .collect();

        match assoc_type {
            "left" => AssocType::Left(symbols),
            "right" => AssocType::Right(symbols),
            "nonassoc" => AssocType::NonAssoc(symbols),
            _ => unreachable!()
        }
    }

    /// 解析文法相关
    /// rules = { rule_decl* }
    fn parse_rules(rules: Pair<Rule>) -> Vec<Production> {
        rules.into_inner()
            .map(|x| Self::parse_rule_decl(x))
            .collect()
    }

    /// 单个推导式声明
    fn parse_rule_decl(rule_decl: Pair<Rule>) -> Production {
        let mut pairs = rule_decl.into_inner();
        // pairs.for_each(|pair| println!("{:?}", pair));
        let name = pairs.next().unwrap().as_str().to_owned();
        let mut production = Production {
            name,
            rules: Vec::new(),
        };
    
        // pairs 内层是 prec_production
        for pair in pairs.into_iter() {
            let mut symbols: Vec<ConfigSymbol> = Vec::new();
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
                        Rule::symbol => symbols.push(Self::parse_symbol(item)),
                        Rule::action => action = Some(item.as_str().to_string()),
                        _ => unreachable!()
                    }
                }
            }


            production.rules.push((symbols, action, prec_directive));
        }
        production
    }

    fn parse_user_code(&mut self, rules: Pair<Rule>) -> String {
        rules.as_str().to_owned()
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
pub fn get_grammar(config: &GrammarConfig) -> (Grammar<usize>, Vec<Option<SymbolMeta>>, Vec<ProdMeta>) {

    let mut grammar = Grammar::new(0);

    let non_terminal: HashMap<String, usize> = config.productions.iter().enumerate()// 非终结符Name -> ID
        .map(|(idx, production)| (production.name.clone(), idx))
        .collect();

    // 注意token_map是无序的，仅用于查询
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
                    let symbol = symbol.content.as_str();
                    if non_terminal.contains_key(symbol) { // 非终结符
                        Symbol::NonTerminal(non_terminal[symbol]) // 查Rule ID
                    } else { // 终结符
                        let tid = *token_map.get(symbol).unwrap_or_else(|| panic!("No Such Token {}", symbol));
                        let meta = token_meta[tid].as_ref().unwrap(); // token_map查得到一定非None
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

fn build_token_map(config: &GrammarConfig) -> (Vec<Option<SymbolMeta>>, HashMap<String, usize>) {
    let mut id_factory = IncIDFactory::new(SYMBOL_ID_BEGIN);
    // productions -> rules -> vec<String> -> Symbols -> Single Symbols

    let singles: IndexSet<_> = config.productions.iter()
        .map(|x| &x.rules)   // 取rules
        .flat_map(|x| x.iter().map(|(rule, _, _)| rule))// 取vec<String>
        .flatten() // 展平成为Symbol数组
        .filter(|x| x.is_single()) // 过滤非single的符号，非终结符一定不会被single匹配
        .map(|x| x.content.as_str())// 取出content
        .collect() // set去重
        ;

    let symbol_sz = SYMBOL_ID_BEGIN + config.tokens.len(); // 所有的符号个数，假设Token不重复声明

    let mut token_meta: Vec<Option<SymbolMeta>> = vec![None; symbol_sz];
    let mut token_map: HashMap<String, usize> = HashMap::new(); // 不遍历，不需要保序

    // 处理%token声明
    for tok in config.tokens.iter() {
        // %token声明的都是多字符的，单字符直接报错
        if is_single_symbol(tok.as_str()) {
            panic!("Unsupported %token declaration: {}", tok);
        }

        // 获取ID
        let id = id_factory.next_id();

        // 重复的就忽略，给一个警告
        if token_map.contains_key(tok.as_str()) {
            eprintln!("Duplicate %token: {}", tok);
            continue;
        }

        token_map.insert(tok.clone(), id); // 保存ID

        // 保存meta
        let symbol_meta = SymbolMeta::new(id, tok.clone());
        token_meta[id] = Some(symbol_meta);
    }




    // 已经都是无重复单字符符号了
    for symbol in singles {
        let id = single2id(symbol);
        token_map.insert(symbol.to_owned(), id);
        // 设置为单字符
        let mut symbol_meta = SymbolMeta::new(id, symbol.to_owned());
        symbol_meta.is_single = true;

        token_meta[id] = Some(symbol_meta);
    }

    // 设置结核性和优先级
    for (idx, assoc) in config.assoc.iter().enumerate() {
        let assoc_type = match assoc {
            AssocType::Right(_) => Assoc::Right,
            AssocType::Left(_) => Assoc::Left,
            AssocType::NonAssoc(_) => Assoc::NonAssoc,
        };
        assoc.unwrap().iter().for_each(|x| {
            let token_id = *token_map.get(x).unwrap_or_else(|| panic!("No Such Token {}", x));
            let meta = token_meta[token_id].as_mut().unwrap();
            meta.priority = idx;
            meta.assoc = assoc_type;

        });
    }

    (token_meta, token_map)
}

/// 是否是单字符symbol，比如'+'
fn is_single_symbol(symbol: &str) -> bool {
    symbol.starts_with("'") && symbol.ends_with("'")
}

/// 获取但字符symbol的id，如果返回ASCII码
fn single2id(symbol: &str) -> usize {
    let symbol = unescape(&symbol[1usize..symbol.len() - 1]).unwrap();
    // 要求单字符必须是单个的，否则报错
    assert_eq!(symbol.len(), 1, "unsupported symbol '{}'", symbol);
    symbol.chars().next().unwrap() as usize
}
