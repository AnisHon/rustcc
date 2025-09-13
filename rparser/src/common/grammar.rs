//!
//! date: 2025/8/26
//! author: anishan
//!
//! 文法相关的类型
//! 文法结构 Symbol(T|NT) -> Rule(vec<SymbolVec>|Epsilon) -> Grammar(Vec<RuleVec>)
//! 文法的额外信息由外部维护
//!
//! # Contents
//! - 文法基础类型 RuleID SymbolID SymbolVec RuleVec
//! - 文法组合类型 ProdMeta Grammar Symbol...
//!

use std::fmt::Debug;
use std::hash::Hash;
use crate::common::grammar::Symbol::{NonTerminal, Terminal};

pub type RuleID = usize;

pub type SymbolID = usize;

pub type SymbolVec<T> = Vec<Symbol<T>>;  // 表示一个推导式

pub type RuleVec<T> = Vec<Rule<T>>;


pub trait SymbolBound: Clone + Debug + Ord + PartialOrd + Eq + PartialEq + Hash {}
impl<T> SymbolBound for T where T: Clone + Debug + Ord + PartialOrd + Eq + PartialEq + Hash {}


/// 推导式额外信息
#[derive(Clone, Debug)]
pub struct ProdMeta {
    pub id: RuleID,      // ID
    pub alter: usize,    // 在grammar alter的索引
    pub name: String,    // 推导式的名字
    pub assoc: Assoc,  // 结合性
    pub priority: usize, // 优先级
    pub action: Option<String>, // 动作
    pub len: usize
}

impl ProdMeta {
    pub fn new(id: RuleID, alter: usize, len: usize, name: String) -> Self {
        Self { id, name, alter, assoc: Assoc::None, priority: 0, action: None, len }
    }
}

/// 文法结合性
/// Left 左结合，同优先级优先规约reduce
/// Right 右结合，同优级优先移入shift
/// NonAssoc 无结合 同优先级下不允许结合
/// None 未指定结合性质
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug, Copy)]
pub enum Assoc {
    Left,
    Right,
    NonAssoc, // 无结合性
    None // 无任何特性
}

/// 推导式符号额外信息，
#[derive(Clone, Debug)]
pub struct SymbolMeta {
    pub id: SymbolID,   // ID
    pub content: String,   // 终结符内容
    pub assoc: Assoc,  // 是否右结合
    pub priority: usize // 优先级
}

impl SymbolMeta {
    pub fn new(id: SymbolID, content: String) -> Self {
        Self { id, content, assoc: Assoc::NonAssoc, priority: 0 }
    }
}


/// 单个符号类型
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Symbol<T: SymbolBound> {
    Terminal(T),
    NonTerminal(RuleID),
}

/// 用于lookahead或者follow的Symbol类型，包含结束符和终结符
#[derive(PartialOrd, PartialEq, Eq, Ord, Clone, Hash, Debug)]
pub enum EndSymbol<T: SymbolBound> {
    End,
    Symbol(T),
}

/// 用于first set等需要空符号的场景，包含空和终结符
#[derive(PartialOrd, PartialEq, Eq, Ord, Clone, Debug)]
pub enum EpsilonSymbol<T: SymbolBound> {
    Epsilon,
    Symbol(T),
}

impl<T: SymbolBound> EpsilonSymbol<T> {

    /// 解包会抛出异常
    /// # Panics
    /// 对Epsilon解包会导致panic
    pub fn unwrap(&self) -> T {
        match self {
            EpsilonSymbol::Epsilon => panic!("Unwarp Epsilon"),
            EpsilonSymbol::Symbol(x) => x.clone()
        }
    }
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

impl<T: SymbolBound> Rule<T> {
    pub fn unwrap_expr(&self) -> &SymbolVec<T> {
        match self {
            Rule::Epsilon => panic!("Rule Is Epsilon"),
            Rule::Expression(x) => x
        }
    }
}

/// 文法规则
/// # Members
/// - rules: 推导式表，rule_id -> RuleVec -> Rule | Rule -> Symbols
/// - start_rule: 入口推导式
///
#[derive(Debug)]
pub struct Grammar<T: SymbolBound> {
    rules: Vec<Option<RuleVec<T>>>,
    start_rule: RuleID,
}

impl<T: SymbolBound> Grammar<T> {
    pub fn new(start_rule: RuleID) -> Self {
        Self {
            rules: Vec::new(),
            start_rule,
        }
    }

    /// 获取rule
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
    
    pub fn add_rule(&mut self, rule_id: RuleID, rule: RuleVec<T>) {
        if rule_id >= self.rules.len() {
            self.rules.resize(rule_id + 1, None);
        }
        
        assert!(self.rules[rule_id].is_none());     // 不允许覆盖
        
        self.rules[rule_id] = Some(rule);
    }
}

