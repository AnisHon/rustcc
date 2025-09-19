//!
//! LR1构造器
//!

use crate::common::grammar::{EndSymbol, EpsilonSymbol, Grammar, Rule, RuleID, RuleVec, Symbol, SymbolBound};
use crate::common::lr_type::{LRItem, LookaheadItemSet, LookaheadStateMap, Transitions};
use crate::util::first_set::{build_first, calc_suffix_first_set};
use crate::util::set_utils;
use common::utils::id_util::IncIDFactory;
use common::utils::unique_id_factory::UniqueIDFactory;
use indexmap::IndexMap;
use std::collections::{BTreeMap, BTreeSet, VecDeque};


/// 只构建DFA状态机不检查冲突
pub struct LR1Builder<'a, T: SymbolBound> {
    grammar: &'a Grammar<T>,
    id_factory: IncIDFactory,
    first_map: IndexMap<RuleID, BTreeSet<EpsilonSymbol<T>>>
}


impl<'a, T: SymbolBound> LR1Builder<'a, T> {
    pub fn new(grammar: &'a Grammar<T>) -> Self {
        let first_map = build_first(grammar);
        Self {
            grammar,
            id_factory: IncIDFactory::new(0),
            first_map
        }
    }
    
    fn get_expr(&self, rule_id: RuleID, alter_idx: usize) -> &Rule<T> {
        &self.get_rule(rule_id)[alter_idx]
    }

    /// 工具方法，获取rule，失败触发panic
    fn get_rule(&self, rule_id: RuleID) -> &RuleVec<T> {
        self.grammar.get_rule(rule_id).unwrap_or_else(|| panic!("rule id {} not found", rule_id))
    }

    /// 跳过next_symbol，计算 first_set \[A ->x·BCx, xx\] FIRST(Cx xx) 跳过了B计算Cx而是BCx
    fn calc_lookahead(&self, item: &LRItem, lookahead: &BTreeSet<EndSymbol<T>>) -> BTreeSet<EndSymbol<T>> {
        let (rule_id, alter_idx) = item.rule;
        let expr = self.get_expr(rule_id, alter_idx).unwrap_expr();

        let (mut result, nullable) = calc_suffix_first_set(expr.iter().skip(item.pos + 1), &self.first_map);

        if nullable {
            result.extend(lookahead.iter().cloned());
        }

        result
    }
    

    /// 项目集闭包
    fn item_closure(&mut self, item_set: &mut LookaheadItemSet<T>) {
        let lookahead_map = &mut item_set.lookahead_map;
        let closure_set = &mut item_set.core_set; // 集合初始化默认元素
        let mut queue: VecDeque<LRItem> = VecDeque::from_iter(closure_set.iter().cloned()); // 初始化队列，不会压入重复元素

        while !queue.is_empty(){
            let item = queue.pop_front().unwrap();
            let next_symbol = item.next_symbol(self.grammar);
            let lookahead = lookahead_map
                .entry(item.clone())
                .or_default();

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
            let items: Vec<_> = (0..alter_rules.len())
                .map(|alter_idx| LRItem::new(rule_id, alter_idx))
                .collect();

            // 跳过next_symbol，计算当前items的闭包
            let next_lookahead = self.calc_lookahead(&item, lookahead);
            // 拓展非终结符
            for item in items {
                let item_lookahead = lookahead_map.entry(item.clone())
                    .or_default();

                let changed = set_utils::extend(item_lookahead, next_lookahead.iter());
                
                // 新项目压入队列，继续闭包
                if !closure_set.contains(&item) || changed {  // 保证不压入重复元素
                    closure_set.insert(item.clone());
                    queue.push_back(item);  // 入队
                }


            }
        }
    }

    /// 项目集go操作
    fn item_goto(&mut self, item_set: LookaheadItemSet<T>, symbol: Symbol<T>) -> LookaheadItemSet<T> {
        let mut next_item_set = LookaheadItemSet { core_set: BTreeSet::new(), lookahead_map: BTreeMap::new() };
        next_item_set.core_set.extend(item_set.core_set.into_iter().map(|item| {
            assert!(item.next_symbol(self.grammar).is_some()); // 非规约项目
            assert_eq!(item.next_symbol(self.grammar).unwrap(), symbol); // 下一个符号是当前符号
            let lookahead = item_set.lookahead_map.get(&item).unwrap().clone();
            let next_item = item.move_next(self.grammar);
            next_item_set.lookahead_map.insert(next_item.clone(), lookahead); // 继承lookahead
            next_item
        }));

        // 闭包
        self.item_closure(&mut next_item_set);
        next_item_set
    }

    /// 获取下一个转移符号，并分类
    fn item_shift_symbols(&self, item_set: &'a LookaheadItemSet<T>) -> BTreeMap<Symbol<T>, LookaheadItemSet<T>> {
        let mut symbols_table = BTreeMap::new();
        for item in item_set.core_set.iter() {
            let symbol = match item.next_symbol(self.grammar) {
                None => continue,
                Some(x) => x
            };
            let next_item_set = symbols_table.entry(symbol)
                .or_insert_with(|| LookaheadItemSet { core_set: BTreeSet::new(), lookahead_map: BTreeMap::new() });
            next_item_set.core_set.insert(item.clone());
            next_item_set.lookahead_map.insert(item.clone(), item_set.lookahead_map[item].clone());
        }
        symbols_table
    }

    /// 获取初始集合
    fn init_item_set(&mut self) -> LookaheadItemSet<T> {
        let start_rule_id = self.grammar.get_start_rule();
        let alter_sz = self.grammar.get_rule(start_rule_id).unwrap().len();

        // 初始化项目集
        let items: BTreeSet<_> = (0..alter_sz).map(|idx| LRItem::new(start_rule_id, idx)).collect();
        let lookahead_map = items.iter().cloned()// 初始化结束符
            .map(|item| (item, BTreeSet::from([EndSymbol::End]))).collect();
        let mut item_set = LookaheadItemSet {
            core_set: items,
            lookahead_map
        };
        self.item_closure(&mut item_set);
        item_set
    }

    ///
    /// 构建LR1状态映射表LR1转移表和
    ///
    /// # Returns
    /// - `usize`: 初始状态
    /// - `Transitions`: 转移表
    /// - `LookaheadStateMap`: 项目集表
    pub fn build_table(&mut self) -> (LookaheadStateMap<T>, Transitions<T>, usize) {
        let init_set = self.init_item_set();
        let mut queue = VecDeque::from(vec![init_set.clone()]);
        let mut items2id_table = IndexMap::new(); // item_set -> item_id
        let mut lr1_table = Vec::new();

        while !queue.is_empty() {
            // 初始项集
            let item_set = queue.pop_front().unwrap();
            // 初始项集ID
            let items_id = *items2id_table
                .entry(item_set.clone())
                .or_insert_with(|| self.id_factory.next_id());

            // 根据下一个转移符号进行分类，得到一个 symbol -> items的映射，过滤规约项目
            let symbol_table = self.item_shift_symbols(&item_set);

            // 对分类进行goto操作
            for (symbol, items) in symbol_table {
                // GOTO(I, X)得到新项目集
                let goto_set = self.item_goto(items, symbol.clone());

                // 有可能已经出现过了，查表或分配ID
                let goto_set_id = *items2id_table.entry(goto_set.clone()).or_insert_with(|| {
                    queue.push_back(goto_set.clone());
                    self.id_factory.next_id()
                });

                // 记录转移
                lr1_table.push((items_id, symbol, goto_set_id));
            }
        }

        // 获取初始状态
        let init_state = items2id_table[&init_set];
        let id2items_table = items2id_table.into_iter()
            .map(|(k, v)| (v, k))
            .collect();
        (id2items_table, lr1_table, init_state)
    }
}



