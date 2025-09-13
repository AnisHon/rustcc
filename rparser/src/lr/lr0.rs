//!
//! date: 2025/8/26
//! author: anishan
//!
//! LR0构造器
//!

use crate::common::grammar::{Grammar, RuleID, RuleVec, Symbol, SymbolBound};
use crate::common::lr_type::LRItem;
use common::utils::id_util::IncIDFactory;
use common::utils::unique_id_factory::UniqueIDFactory;
use indexmap::IndexMap;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub type LR0ItemSet = BTreeSet<LRItem>;

/// 只构建DFA状态机不检查冲突
pub struct LR0Builder<'a, T: SymbolBound> {
    grammar: &'a Grammar<T>,
    id_factory: IncIDFactory,
}

impl<'a, T: SymbolBound> LR0Builder<'a, T> {

    pub fn new(grammar: &'a Grammar<T>) -> Self {
        Self { grammar, id_factory: IncIDFactory::new(0) }
    }


    /// 工具方法，获取rule，失败触发panic
    fn get_rule(&self, rule_id: RuleID) -> &RuleVec<T> {
        self.grammar.get_rule(rule_id).unwrap_or_else(|| panic!("rule id {} not found", rule_id))
    }

    /// 项目集闭包
    fn item_closure(&mut self, items: LR0ItemSet) -> LR0ItemSet {
        let mut closure_set: LR0ItemSet = items.clone(); // 集合初始化默认元素
        let mut queue: VecDeque<LRItem> = VecDeque::from_iter(items.into_iter()); // 初始化队列，不会压入重复元素

        while !queue.is_empty(){
            let item = queue.pop_front().unwrap();
            let next_symbol = item.next_symbol(self.grammar);

            // 下一个symbol
            let symbol = match next_symbol {
                None => continue, // 规约项目无需闭包
                Some(x) => x
            };

            // symbol是非终结符
            let rule_id = match symbol {
                Symbol::Terminal(_) => continue, // 终结符无需闭包
                Symbol::NonTerminal(rule_id) => rule_id
            };

            // 非终结符
            let alter_rules = self.get_rule(rule_id);
            let items = (0..alter_rules.len())
                .map(|alter_idx| LRItem::new(rule_id, alter_idx));

            // 拓展非终结符
            for item in items {
                // 新项目压入队列，继续闭包
                if !closure_set.contains(&item) {  // 保证不压入重复元素
                    closure_set.insert(item.clone());
                    queue.push_back(item);  // 入队
                }
            }

        }

        closure_set
    }

    /// 项目集go操作
    /// 
    /// # Arguments 
    /// - 'items': 内部的LR0Item项目集，必须全是经过symbol转移的，该函数不负责过滤
    /// - 'symbol': 下一次转移符号
    fn item_goto(&mut self, items: BTreeSet<LRItem>, symbol: Symbol<T>) -> LR0ItemSet {
        let items: BTreeSet<_> = items.into_iter()// 移动GO操作
            .map(|item| {
                assert!(item.next_symbol(self.grammar).is_some()); // 非规约项目
                assert_eq!(item.next_symbol(self.grammar).unwrap(), symbol); // 下一个符号是当前符号
                item.move_next(self.grammar)
            })
            .collect();

        if items.is_empty() {
            BTreeSet::new()
        } else {
            self.item_closure(items)
        }
    }

    /// 获取项目集转移符号
    fn item_symbols(&self, items: &LR0ItemSet) -> BTreeMap<Symbol<T>, BTreeSet<LRItem>> {
        let mut symbols_table = BTreeMap::new();
        for item in items.iter() {
            let symbol = match item.next_symbol(self.grammar) {
                None => continue,
                Some(x) => x
            };
            symbols_table.entry(symbol).or_insert(BTreeSet::new()).insert(item.clone());
        }
        symbols_table
    }

    /// 获取初始集合
    fn init_item_set(&mut self) -> LR0ItemSet {
        let start_rule_id = self.grammar.get_start_rule();
        let alter_sz = self.grammar.get_rule(start_rule_id).unwrap().len();
        let start: BTreeSet<_> =
            (0..alter_sz).map(|idx| LRItem::new(start_rule_id, idx)).collect();

        self.item_closure(start)
    }

    /// 构建表
    /// 
    /// # Returns
    /// - 'id2items_table': id映射表items_id -> item_set
    /// - 'lr0_table': LR0表，使用三元组表示(items_id, symbol, items_id)
    /// todo 太复杂，别名
    pub fn build_table(mut self) -> (IndexMap<usize, LR0ItemSet>, Vec<(usize, Symbol<T>, usize)>, usize) {
        let init_set = self.init_item_set();
        let mut queue = VecDeque::from(vec![init_set.clone()]);
        let mut items2id_table = BTreeMap::new(); // item_set -> item_id
        let mut lr0_table = Vec::new();
        
        while !queue.is_empty() {
            let item_set = queue.pop_front().unwrap();
            let items_id = *items2id_table
                .entry(item_set.clone())
                .or_insert_with(|| self.id_factory.next_id());
        
            let symbol_table = self.item_symbols(&item_set);

            // 转移边symbol，对应的转移集合items
            for (symbol, items) in symbol_table {
                let goto_set = self.item_goto(items, symbol.clone());
                let goto_set_id = *items2id_table.entry(goto_set.clone()).or_insert_with(|| {
                    queue.push_back(goto_set.clone());
                   self.id_factory.next_id()
                });

                lr0_table.push((items_id, symbol, goto_set_id));
            }
        }

        let init_state = items2id_table[&init_set];
        let id2items_table: IndexMap<_, _> = items2id_table.into_iter()
            .map(|(k, v)| (v, k))
            .collect();

        (id2items_table, lr0_table, init_state)
    }
}


