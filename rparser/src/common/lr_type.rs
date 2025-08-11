//! 声明LR相关的类型
//! 
use crate::common::grammar::{EndSymbol, Grammar, Rule, RuleID, Symbol, SymbolBound};
use std::collections::{BTreeSet, HashMap};

pub type ItemID = usize;


/// 通用LR项目，引用rule，管理pos
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct LRItem {
    pub rule: (RuleID, usize), //通过rule_id和alter索引引用rule
    pub pos: usize,
}

impl LRItem {

    /// 创建一个LR项目，Lookahead为空
    pub fn new(rule_id: RuleID, idx: usize) -> LRItem {
        LRItem {rule: (rule_id, idx), pos: idx}
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

    /// 向后移动
    pub fn move_next<T: SymbolBound>(&self, grammar: &Grammar<T>) -> Self {
        assert!(!self.is_reduced(grammar)); // 规约项目不能在移动

        Self {
            rule: self.rule,
            pos: self.pos + 1,
        }
    }

}
