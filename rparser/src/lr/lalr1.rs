use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use common::utils::id_util::IncIDFactory;
use crate::common::grammar::{EndSymbol, EpsilonSymbol, Grammar, Rule, RuleID, RuleVec, Symbol, SymbolBound};
use crate::common::lr_type::{LRItem, LookaheadItemSet};
use crate::lr::lr0::{LR0Builder, LR0ItemSet};
use crate::lr::lr1::{LR1Builder};
use crate::util::first_set::{build_first, FirstMap};
use crate::util::set_utils::extend;

pub struct LALR1Builder<'a, T: SymbolBound> {

    grammar: &'a Grammar<T>,
    first_map: FirstMap<T>,
    id2item_map: HashMap<usize, LR0ItemSet>,
    transition_table: Vec<(usize, Symbol<T>, usize)>,
    init_state: usize,
}

/// 使用LR0传播算法构建LALR
impl<'a, T: SymbolBound> LALR1Builder<'a, T> {

    pub fn new(grammar: &'a Grammar<T>) -> Self {
        let first_map = build_first(grammar);
        let (id2item_map, transition_table, init_state) = LR0Builder::new(grammar).build_table();
        Self {
            grammar,
            first_map,
            id2item_map,
            transition_table,
            init_state,
        }
    }


    /// 构建Table
    pub fn build_table(self) -> (HashMap<usize, LookaheadItemSet<T>>, Vec<(usize, Symbol<T>, usize)>, usize) {
        let (id_state_item_map, state_item_id_map) = self.item_state_map();
        let (graph, mut lookahead_item_set_map) = self.init_propagation(&state_item_id_map);

        let mut work_list = VecDeque::from_iter(0..id_state_item_map.len()); // 所有item都进入队列开始传播
        let mut visited: HashSet<usize> = work_list.iter().copied().collect();


        while work_list.is_empty() {
            let id = work_list.pop_front().unwrap();
            visited.remove(&id); // 工作队列弹出

            let (state, item) = &id_state_item_map[id];
            let lookahead_item_set = lookahead_item_set_map.get_mut(state).unwrap();
            let lookahead_set = lookahead_item_set.lookahead_map.get_mut(item).unwrap().clone(); // borrow checker 没办法

            // 遍历传播边
            for &prop_id in &graph[id] {
                // 被传播的lookahead
                let (prop_state, prop_item) = &id_state_item_map[prop_id];
                let prop_lookahead_item_set = lookahead_item_set_map.get_mut(prop_state).unwrap();
                let prop_lookahead_set = prop_lookahead_item_set.lookahead_map.get_mut(prop_item).unwrap();

                let changed = extend(prop_lookahead_set, lookahead_set.iter());

                if !changed { // 没改变
                    continue
                }
                // 如果发生改变进入下一轮传播
                if !visited.contains(&prop_id) { // 工作队列去重
                    work_list.push_back(prop_id);
                }

            }
        }

        (lookahead_item_set_map, self.transition_table, self.init_state)
    }


    /// 构建 id -> (state, LRItem) 的 Map
    fn item_state_map(&self) -> (Vec<(usize, LRItem)>, HashMap<(usize, LRItem), usize>) {
        let id_state_item_map: Vec<_> = self.id2item_map.iter()
            .map(|(state, item_set)|
                item_set.iter().cloned().map(|item| (state.clone(), item))
            ).flatten()
            .collect();
        let state_item_id_map: HashMap<_, _> = id_state_item_map.iter()
            .cloned().enumerate()
            .map(|(idx, item)| (item, idx))
            .collect();

        (id_state_item_map, state_item_id_map)
    }

    /// 初始化自发lookahead 和 传播边
    /// ### return
    /// Vec<BTreeSet<usize>>，依赖图（邻接表）
    /// Vec<LookaheadItemSet<T>> LALR item set表
    fn init_propagation(&self, state_item_id_map: &HashMap<(usize, LRItem), usize>) -> (Vec<BTreeSet<usize>>, HashMap<usize, LookaheadItemSet<T>>) {
        let mut graph = Vec::new();
        graph.resize_with(state_item_id_map.len(), || BTreeSet::new());

        // 转换为LookaheadItemSet
        let mut id2lookahead_map: HashMap<_, _> = self.id2item_map.iter().map(|(&idx, item_set)| {
            let core_set = item_set.clone();
            let lookahead_map: BTreeMap<_, BTreeSet<EndSymbol<T>>> = core_set.iter()
                .cloned()
                .map(|x| (x, BTreeSet::new())).collect();
            (idx, LookaheadItemSet {core_set, lookahead_map})
        }).collect();

        // 处理闭包传播和自发lookahead
        for ((state_id, item), id) in state_item_id_map {
            let (lookahead, nullable) = self.calc_lookahead(item);
            let items = self.once_closure(item);

            // 自发
            for closure_item in items {
                if nullable { // 传播
                    let closure_id = state_item_id_map[&(*state_id, item.clone())];
                    graph[*id].insert(closure_id); // 添加边
                }

                // 设置自发lookahead
                let lookahead_item_set = id2lookahead_map.get_mut(state_id).unwrap();
                let lookahead_set = lookahead_item_set.lookahead_map.get_mut(&closure_item).unwrap();
                lookahead_set.extend(lookahead.iter().cloned())
            }
        }

        // 处理转移传播
        for (from, _, to) in self.transition_table.iter() {
            let from_item_set = &self.id2item_map[&from];
            for item in from_item_set {
                // 获取转移项目
                let next_item = match item.is_reduced(self.grammar) {
                    true => continue,
                    false => item.clone().move_next(self.grammar)
                };

                let id = state_item_id_map[&(*from, item.clone())]; // 当前项目id
                let next_id = state_item_id_map[&(*to, next_item)]; // 转移到的项目id
                graph[id].insert(next_id); // 添加边
            }
        }

        // 初始种子（终结符）
        // 初始item
        let start_rule_id = self.grammar.get_start_rule();
        let alter_sz = self.grammar.get_rule(start_rule_id).unwrap().len();
        let init_items: BTreeSet<_> = (0..alter_sz).map(|idx| LRItem::new(start_rule_id, idx)).collect();
        for item in init_items {
            let lookahead_item_set = id2lookahead_map.get_mut(&self.init_state).unwrap();
            let lookahead_set = lookahead_item_set.lookahead_map.get_mut(&item).unwrap();
            lookahead_set.insert(EndSymbol::End); // 结束符，自发生成
        }

        (graph, id2lookahead_map)
    }

    fn get_expr(&self, rule_id: RuleID, alter_idx: usize) -> &Rule<T> {
        &self.get_rule(rule_id)[alter_idx]
    }
    /// 工具方法，获取rule，失败触发panic
    fn get_rule(&self, rule_id: RuleID) -> &RuleVec<T> {
        self.grammar.get_rule(rule_id).expect(format!("rule id {} not found", rule_id).as_str())
    }


    /// 跳过next_symbol，计算 first_set \[A ->x·BCx\] FIRST(Cx) 跳过了B计算Cx而是BCx
    /// ### return
    /// BTreeSet<EndSymbol<T>>: 自发lookahead
    /// bool: nullable
    fn calc_lookahead(&self, item: &LRItem) -> (BTreeSet<EndSymbol<T>>, bool) {
        let mut spontaneous_lookahead: BTreeSet<EndSymbol<T>> = BTreeSet::new(); // 计算Lookahead是就是计算first

        let (rule_id, alter_idx) = item.rule;
        let expr = self.get_expr(rule_id, alter_idx).unwrap_expr();

        let mut nullable = true;
        for idx in (item.pos + 1)..expr.len() {
            let rule_id = match &expr[idx] {
                Symbol::Terminal(x) => { // 终结符不能推出空，非nullable，停止迭代
                    spontaneous_lookahead.insert(EndSymbol::Symbol(x.clone()));
                    nullable = false;
                    break
                },
                Symbol::NonTerminal(x) => x
            };

            let first_set = &self.first_map[rule_id];
            spontaneous_lookahead.extend(
                first_set.iter()
                    .filter(|&x| x.ne(&EpsilonSymbol::Epsilon))
                    .map(|x| EndSymbol::Symbol(x.unwrap_symbol()))
            );

            if !first_set.contains(&EpsilonSymbol::Epsilon) { // 不能推出空退出
                nullable = false;
                break;
            }
        }

        (spontaneous_lookahead, nullable)
    }

    /// 一次闭包，也就是将item传入后向后闭包一次
    fn once_closure(&self, item: &LRItem) -> BTreeSet<LRItem> {
        let rule_id = match item.next_symbol(self.grammar).unwrap() {
            Symbol::NonTerminal(x) => x,
            _ => unreachable!()
        };

        let len = self.get_rule(rule_id).len();
        let items: BTreeSet<LRItem> = (0..len).map(|idx| LRItem::new(rule_id, idx)).collect();
        items
    }


}

/// 使用LR1合并算法构建LALR
pub struct AdvancedLALR1Builder<T: SymbolBound> {
    id2item_map: HashMap<usize, LookaheadItemSet<T>>,
    transition_table: Vec<(usize, Symbol<T>, usize)>,
    init_state: usize,
    id_factory: IncIDFactory,
}

impl<T: SymbolBound> AdvancedLALR1Builder<T> {
    pub fn new(grammar: &Grammar<T>) -> Self {
        let (id2item_map, transition_table, init_state) = LR1Builder::new(grammar).build_table();
        Self {
            id2item_map,
            transition_table,
            init_state,
            id_factory: IncIDFactory::new(0),
        }
    }
}
