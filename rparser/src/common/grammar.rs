use std::fmt::Debug;
use crate::common::grammar::Symbol::{NonTerminal, Terminal};

pub type RuleID = usize;

pub type SymbolVec<T> = Vec<Symbol<T>>;  // 表示一个推导式

pub type RuleVec<T> = Vec<Rule<T>>;


pub trait SymbolBound: Clone + Debug + Ord + PartialOrd + Eq + PartialEq {}
impl<T> SymbolBound for T where T: Clone + Debug + Ord + PartialOrd + Eq + PartialEq {}

#[derive(Clone, Debug)]
pub struct RuleMeta {
    pub name: String,   // 推导式的名字
    pub optional: bool, // 是否是可选的，是否能推导出空
}

/// 单个符号类型
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Symbol<T: SymbolBound> {
    Terminal(T),
    NonTerminal(RuleID),
}

/// 用于lookahead follow的Symbol类型，结束符号和终结符号
#[derive(PartialOrd, PartialEq, Eq, Ord)]
pub enum EndSymbol<T: SymbolBound> {
    End,
    Symbol(T),
}
/// 用于first set等需要空符号的场景，只包含空和终结符
#[derive(PartialOrd, PartialEq, Eq, Ord, Clone, Debug)]
pub enum EpsilonSymbol<T: SymbolBound> {
    Epsilon,
    Symbol(T),
}


impl<T: SymbolBound> Debug for Symbol<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Terminal(chr) => write!(f, "{:?}", chr),
            NonTerminal(id) => write!(f, "{}", id),
        }

    }
}


/// 推导式类型
#[derive(Clone, Debug)]
pub enum Rule<T: SymbolBound> {
    Epsilon, // 空推导式
    Expression(SymbolVec<T>),
}

/// 文法规则
/// ### 成员
/// - rules: 推导式表，rule_id -> RuleVec -> Rule | Rule -> Symbols
/// - rule_meta: 推导式信息表
/// - start_rule: 入口推导式
#[derive(Debug)]
pub struct Grammar<T: SymbolBound> {
    rules: Vec<Option<RuleVec<T>>>,
    rule_meta: Vec<Option<RuleMeta>>,
    start_rule: RuleID,
}

impl<T: SymbolBound> Grammar<T> {
    pub fn new(start_rule: RuleID) -> Self {
        Self {
            rules: Vec::new(),
            rule_meta: Vec::new(),
            start_rule,
        }
    }

    pub fn get_meta(&self, rule_id: RuleID) -> Option<&RuleMeta> {
        self.rule_meta[rule_id].as_ref()
    }
    pub fn get_rule(&self, rule_id: RuleID) -> Option<&RuleVec<T>> {
        assert!(rule_id < self.rules.len());
        self.rules[rule_id].as_ref()
    }

    pub fn get_start_rule(&self) -> RuleID {
        assert!(self.start_rule < self.rules.len() && self.rules[self.start_rule].is_some()); // 必须存在
        self.start_rule
    }
    
    pub fn get_size(&self) -> usize {
        self.rules.len()
    }
    
    pub fn add_rule(&mut self, rule_id: RuleID, rule: RuleVec<T>, mut meta: RuleMeta) {
        if rule_id >= self.rules.len() {
            self.rules.resize(rule_id + 1, None);
        }
        if rule_id >= self.rule_meta.len() {
            self.rule_meta.resize(rule_id + 1, None);
        }

        assert!(self.rules[rule_id].is_none());     // 不允许覆盖
        assert!(self.rule_meta[rule_id].is_none());
        
        self.rules[rule_id] = Some(rule);
        self.rule_meta[rule_id] = Some(meta);
        
    }
    

}

