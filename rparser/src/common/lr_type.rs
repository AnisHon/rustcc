//! date: 2025/5/26
//! author: anishan
//!
//! 声明LR相关的类型
//!
//! # Contents
//! - LRItem LR项目
//! - LookaheadItemSet LRItem的集合，附带Lookahead
//!
//!
//!

use crate::common::grammar::{EndSymbol, Grammar, Rule, RuleID, Symbol, SymbolBound};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;
use indexmap::IndexMap;

/// LR一个状态项目集表
pub type LookaheadStateMap<T> = IndexMap<usize, LookaheadItemSet<T>>;

/// LR转移表
pub type Transitions<T> = Vec<(usize, Symbol<T>, usize)>;

/// action表
pub type ActionTable = Vec<Vec<LRAction>>;

/// goto表
pub type GotoTable = Vec<Vec<Option<usize>>>;

/// 通用LR项目，引用rule，管理pos
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct LRItem {
    pub rule: (RuleID, usize), //通过rule_id和alter索引引用rule
    pub pos: usize,
}

impl LRItem {

    /// 创建一个LR项目，Lookahead为空
    pub fn new(rule_id: RuleID, idx: usize) -> LRItem {
        LRItem {rule: (rule_id, idx), pos: 0}
    }

    fn get_rule<T: SymbolBound>(grammar: &Grammar<T>,rule_id: RuleID, idx: usize) -> &Rule<T> {
        grammar.get_rule(rule_id).unwrap().get(idx).unwrap()
    }

    /// 获取下一个符号，如果是规约串则返回None
    pub fn next_symbol<T: SymbolBound>(&self, grammar: &Grammar<T>) -> Option<Symbol<T>> {
        if self.is_reduced(grammar) {
            None
        } else {
            let (rule_id, idx) = self.rule;
            let expr = Self::get_rule(grammar, rule_id, idx).unwrap_expr();
            Some(expr[self.pos].clone())
        }
    }

    /// 是否是归并项目
    pub fn is_reduced<T: SymbolBound>(&self, grammar: &Grammar<T>) -> bool {
        let (rule_id, idx) = self.rule;

        match Self::get_rule(grammar, rule_id, idx) {
            Rule::Epsilon => true,
            Rule::Expression(x) => self.pos >= x.len()
        }
    }

    /// 是否是初始项目
    pub fn is_start<T: SymbolBound>(&self, grammar: &Grammar<T>) -> bool {
        grammar.get_start_rule() == self.rule.0
    }

    /// 向后移动
    pub fn move_next<T: SymbolBound>(mut self, grammar: &Grammar<T>) -> Self {
        assert!(!self.is_reduced(grammar)); // 规约项目不能在移动
        self.pos += 1;
        self
    }

}

/// lookahead集合，适用于LR1 LALR1 SLR1
/// # members
/// - 'core_set': LRItem核心集
/// - 'lookahead_map': 展望串映射表
///
#[derive(Debug, Clone)]
#[derive(Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct LookaheadItemSet<T: SymbolBound> {
    pub core_set: BTreeSet<LRItem>,
    pub lookahead_map: BTreeMap<LRItem, BTreeSet<EndSymbol<T>>>
}

/// LR表格的规约移入操作
/// 默认为Error，同样作为稀疏供压缩矩阵使用
#[derive(Debug, Clone, PartialEq, Copy, Eq, Hash)]
pub enum LRAction {
    Reduce(usize), // 规约 推导式ID
    Shift(usize),  // 移入 状态ID
    Accept(usize),    // 结束规约 推导式ID
    Error          // 出错Error
}

impl Default for LRAction {
    fn default() -> Self {
        LRAction::Error
    }
}

impl LRAction {
    pub fn is_shift(&self) -> bool{
        match self {
            LRAction::Reduce(_) | LRAction::Accept(_) | LRAction::Error => false,
            LRAction::Shift(_) => true
        }
    }
    
    pub fn unwrap(&self) -> usize {
        *match self {
            LRAction::Reduce(x)
            | LRAction::Shift(x)
            | LRAction::Accept(x) => x,
            LRAction::Error => panic!("Action is Error")
        }
    }
}