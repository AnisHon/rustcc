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

/// LR项目管理器，生成构建维护生成LR项目，以(LRItem, lookahead)为key，保证相同的项目拥有相同的ID
/// ### members
/// item_map: ItemID -> item
/// lookahead_map: ItemID -> lookahead_set 
/// grammar: 文法规则
/// item_idx_map: (item, lookahead) -> id，用于保证LRItem不重复
pub struct LRItemManager<'a, T: SymbolBound> {
    grammar: &'a Grammar<T>,
    item_map: Vec<LRItem>,
    lookahead_map: Vec<Option<BTreeSet<EndSymbol<T>>>>,
    item_idx_map: HashMap<(LRItem, Option<BTreeSet<EndSymbol<T>>>), ItemID> // 复杂KEY，顺序无关 Hash更高效
}

impl<'a, T: SymbolBound> LRItemManager<'a, T> {
    pub fn new(grammar: &'a Grammar<T>) -> Self {
        Self {
            item_map: Vec::new(),
            lookahead_map: Vec::new(),
            grammar,
            item_idx_map: HashMap::new()
        }
    }

    fn get_rule(&self, rule_id: RuleID, idx: usize) -> &Rule<T> {
        self.grammar.get_rule(rule_id).unwrap().get(idx).unwrap()
    }

    /// 创建一个LR0项目
    pub fn lr0_item(&mut self, rule_id: RuleID, idx: usize) -> RuleID {
        let item = LRItem {rule: (rule_id, idx), pos: idx};
        *self.item_idx_map // 检查是否已经存在
            .entry((item.clone() , None))
            .or_insert_with(|| { // 不存在则添加
                self.item_map.push(item);
                self.lookahead_map.push(None); // LR0没有Lookahead
                self.item_map.len() - 1
            })
    }

    pub fn get_item(&self, rule_id: RuleID) -> &LRItem {
        assert!(rule_id < self.item_map.len());
        self.item_map.get(rule_id).unwrap()
    }

    /// 获取下一个符号，如果是规约串则返回None
    pub fn next_symbol(&self, item_id: ItemID) -> Option<Symbol<T>> {
        if self.is_reduced(item_id) {
            None
        } else {
            let item = &self.item_map[item_id];
            let (rule_id, idx) = item.rule;
            let expr = self.get_rule(rule_id, idx).unwrap_expr();
            Some(expr[item.pos].clone())
        }
    }

    /// 是否是归并项目
    pub fn is_reduced(&self, item_id: ItemID) -> bool {
        assert!(item_id < self.item_map.len());
        let item = &self.item_map[item_id];
        let (rule_id, idx) = item.rule;

        match self.get_rule(rule_id, idx) {
            Rule::Epsilon => true,
            Rule::Expression(x) => item.pos >= x.len()
        }
    }

    /// 向后移动
    pub fn move_next(&mut self, item_id: ItemID) -> RuleID {
        assert!(!self.is_reduced(item_id)); // 规约项目不能在移动
        let mut next_item = self.item_map[item_id].clone();
        next_item.pos += 1;

        *self.item_idx_map // 检查是否存在
            .entry((next_item.clone(), self.lookahead_map[item_id].clone()))
            .or_insert_with(|| { // 不存在则构建
                self.item_map.push(next_item);
                let rule_id: RuleID = self.item_map.len() - 1;

                self.lookahead_map.push(self.lookahead_map[item_id].clone()); // goto操作lookahead不变
                rule_id
            })
    }

}


