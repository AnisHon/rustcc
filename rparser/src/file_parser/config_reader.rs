use std::collections::{BTreeSet, HashMap};
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use crate::common::grammar::{Assoc, Grammar, RuleMeta, RuleVec, Symbol, SymbolMeta, SymbolVec};

#[derive(Parser)]
#[grammar = "parser_pest.pest"]
struct BisonParser;

#[derive(Debug)]
pub struct GrammarConfig {
    pub tokens: Vec<String>,
    pub assoc: Vec<AssocType>,
    pub productions: Vec<Production>
}

/// Token类型数组
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

#[derive(Debug)]
pub struct Production {
    pub name: String,
    pub rules: Vec<(Vec<String>, Option<String>)>,
}

pub struct GrammarConfigParser {
    input: String,
    tokens: Vec<String>,
    assoc: Vec<AssocType>,
    productions: Vec<Production>
}

impl GrammarConfigParser {
    pub fn new(input: String) -> Self {
        Self { input, tokens: Vec::new(), assoc: Vec::new(), productions: Vec::new() }
    }

    pub fn parse(mut self) -> GrammarConfig {
        let input = self.input.clone();
        let pairs = BisonParser::parse(Rule::file, input.as_str()).unwrap();



        for pair in pairs {
            self.parse_file(pair);
        }
        GrammarConfig { tokens: self.tokens, assoc: self.assoc, productions: self.productions }
    }

    fn parse_file(&mut self, file: Pair<Rule>) {
        for rule in file.into_inner() {
            match rule.as_rule() {
                Rule::decls => self.parse_decls(rule),
                Rule::rules => self.parse_rules(rule),
                Rule::user_code => self.parse_user_code(rule),
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
                    _ => unreachable!(),
                }
            }
        }

    }

    fn parse_token_decl(&mut self, decl: Pair<Rule>) {
        self.tokens.extend(decl.into_inner().into_iter().map(|x| x.as_str().to_string()));
    }
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

    fn parse_rules(&mut self, rules: Pair<Rule>) {
        for rule_decl in rules.into_inner() {
            let production = Self::parse_rule_decl(rule_decl);
            self.productions.push(production);
        }
    }

    fn parse_rule_decl(rule_decl: Pair<Rule>) -> Production {
        let mut pairs = rule_decl.into_inner();
        // pairs.for_each(|pair| println!("{:?}", pair));
        let mut production = Production {
            name: pairs.next().unwrap().as_str().to_string(),
            rules: Vec::new(),
        };

        for pair in pairs.into_iter() {
            let mut symbols = Vec::new();
            let mut action = None;
            for item in pair.into_inner() {
                match item.as_rule() {
                    Rule::symbol => symbols.push(item.as_str().to_string()),
                    Rule::action => action = Some(item.as_str().to_string()),
                    _ => unreachable!()
                }
            }
            production.rules.push((symbols, action));
        }

        production
    }

    fn parse_user_code(&mut self, rules: Pair<Rule>) {

    }

}



pub fn get_grammar(config: &GrammarConfig) -> (Grammar<usize>, Vec<SymbolMeta>) {

    let mut grammar = Grammar::new(0);

    let non_terminal: HashMap<String, usize> = config.productions.iter().enumerate()// 非终结符Name -> ID
        .map(|(idx, production)| (production.name.clone(), idx))
        .collect();

    // todo 通过lexer声明，确定token ID
    let (token_meta, token_map) = build_token_map(&config, &non_terminal);
    
    // 构建推导式
    for (rule_id, production) in config.productions.iter().enumerate() {
        let mut meta = RuleMeta::new(rule_id, production.name.clone()); // 构建Meta
        let mut rule_vec = RuleVec::new();

        // 遍历所有alter
        for (symbols, action) in production.rules.iter() {
            let action = action.clone();
            let mut priority = 0;
            let mut assoc = Assoc::None;
            let rule = if symbols.is_empty() { // 空 epsilon
                crate::common::grammar::Rule::Epsilon
            } else {
                // 非空，遍历所有symbol
                let symbol_vec: SymbolVec<_> = symbols.into_iter().map(|symbol|{
                    if non_terminal.contains_key(symbol) {
                        Symbol::NonTerminal(non_terminal[symbol]) // 查Rule ID
                    } else {
                        let tid = token_map[symbol];
                        let meta = &token_meta[tid];
                        priority = meta.priority;   // 推导式以最后的终结符为准
                        assoc = meta.assoc;
                        Symbol::Terminal(tid)
                    }
                }).collect();
                crate::common::grammar::Rule::Expression(symbol_vec)
            };

            rule_vec.push(rule);
            meta.action.push(action);
            meta.assoc.push(assoc);
            meta.priority.push(priority);
        }

        grammar.add_rule(rule_id, rule_vec, meta.clone());
    }

    (grammar, token_meta)
}

fn build_token_map(config: &GrammarConfig, non_terminal: &HashMap<String, usize>) -> (Vec<SymbolMeta>, HashMap<String, usize>) {

    let tokens: BTreeSet<_> = config.productions.iter()
        .map(|production| &production.rules)
        .flatten()
        .map(|(symbols, _)| symbols.iter().cloned())
        .flatten()
        .filter(|x| !non_terminal.contains_key(x))
        .collect();

    let mut token_meta: Vec<_> = tokens.into_iter().enumerate()
        .map(|(idx, token)| SymbolMeta::new(idx, token))
        .collect();

    let token_map: HashMap<_, _> = token_meta.iter()
        .map(|x| (x.content.clone(), x.id))
        .collect();

    for (idx, assoc) in config.assoc.iter().enumerate() {
        let is_right = match assoc {
            AssocType::Right(_) => Assoc::Right,
            AssocType::Left(_) => Assoc::Left,
            AssocType::NonAssoc(_) => Assoc::None,
        };
        assoc.unwrap().iter().for_each(|x| {
            let meta = &mut token_meta[token_map[x]];
            meta.priority = idx;
            meta.assoc = is_right;
        });
    }


    (token_meta, token_map)
}


