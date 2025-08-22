use crate::common::grammar::{EndSymbol, EpsilonSymbol, Grammar, Rule, RuleID, RuleMeta, RuleVec, Symbol, SymbolBound};
use crate::common::lr_type::{LRItem, LookaheadItemSet};
use crate::lr::lr0::{LR0Builder, LR0ItemSet};
use crate::lr::lr1::LR1Builder;
use crate::util::first_set::{build_first, FirstMap};
use crate::util::set_utils::extend;
use common::utils::id_util::IncIDFactory;
use common::utils::unique_id_factory::UniqueIDFactory;
use indexmap::IndexMap;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

pub struct LALR1Builder<'a, T: SymbolBound> {

    grammar: &'a Grammar<T>,
    first_map: FirstMap<T>,
    id2item_map: IndexMap<usize, LR0ItemSet>,
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
    pub fn build_table(self) -> (IndexMap<usize, LookaheadItemSet<T>>, Vec<(usize, Symbol<T>, usize)>, usize) {
        let (id_state_item_map, state_item_id_map) = self.item_state_map();
        let (graph, mut lookahead_item_set_map) = self.init_propagation(&state_item_id_map);

        let mut work_list = VecDeque::from_iter(0..id_state_item_map.len()); // 所有item都进入队列开始传播
        let mut visited: HashSet<usize> = work_list.iter().copied().collect();

        println!("{:?}", graph.iter().enumerate().collect::<Vec<_>>());

        while !work_list.is_empty() {
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
    fn item_state_map(&self) -> (Vec<(usize, LRItem)>, IndexMap<(usize, LRItem), usize>) {
        let id_state_item_map: Vec<_> = self.id2item_map.iter()
            .map(|(state, item_set)|
                item_set.iter().cloned().map(|item| (state.clone(), item))
            ).flatten()
            .collect();
        let state_item_id_map: IndexMap<_, _> = id_state_item_map.iter()
            .cloned().enumerate()
            .map(|(idx, item)| (item, idx))
            .collect();

        (id_state_item_map, state_item_id_map)
    }

    /// 初始化自发lookahead 和 传播边
    /// ### return
    /// Vec<BTreeSet<usize>>，依赖图（邻接表）
    /// Vec<LookaheadItemSet<T>> LALR item set表
    fn init_propagation(&self, state_item_id_map: &IndexMap<(usize, LRItem), usize>) -> (Vec<BTreeSet<usize>>, IndexMap<usize, LookaheadItemSet<T>>) {
        let mut graph = Vec::new();
        graph.resize_with(state_item_id_map.len(), || BTreeSet::new());

        // 转换为LookaheadItemSet
        let mut id2lookahead_map: IndexMap<_, _> = self.id2item_map.iter().map(|(&idx, item_set)| {
            let core_set = item_set.clone();
            let lookahead_map: BTreeMap<_, BTreeSet<EndSymbol<T>>> = core_set.iter()
                .cloned()
                .map(|x| (x, BTreeSet::new())).collect();
            (idx, LookaheadItemSet {core_set, lookahead_map})
        }).collect();

        // 处理闭包传播和自发lookahead
        for ((state_id, item), id) in state_item_id_map {
            // 规约项目不会主动传播
            if item.is_reduced(self.grammar) {
                continue;
            }
            let (lookahead, nullable) = self.calc_lookahead(item);
            let items = match self.once_closure(item) {
                Some(x) => x,
                None => continue,
            };

            // 自发
            for closure_item in items {
                if nullable { // 传播
                    let closure_id = state_item_id_map[&(*state_id, closure_item.clone())];
                    graph[*id].insert(closure_id); // 添加边
                }

                // 设置自发lookahead
                let lookahead_item_set = id2lookahead_map.get_mut(state_id).unwrap();
                let lookahead_set = lookahead_item_set.lookahead_map.get_mut(&closure_item).unwrap();
                lookahead_set.extend(lookahead.iter().cloned())
            }
        }

        // 处理转移传播
        for (from, symbol, to) in self.transition_table.iter() {
            let from_item_set = &self.id2item_map[from];
            for item in from_item_set {
                let next_symbol = match item.next_symbol(self.grammar) {
                    None => continue,
                    Some(x) => x
                };

                // 要求相同转移边
                if symbol.ne(&next_symbol) {
                    continue
                }

                // 获取转移项目
                let next_item = item.clone().move_next(self.grammar);

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
    fn once_closure(&self, item: &LRItem) -> Option<BTreeSet<LRItem>> {
        let next_symbol = match item.next_symbol(self.grammar) {
            Some(x) => x,
            None => return None
        };
        let rule_id = match next_symbol {
            Symbol::NonTerminal(x) => x,
            _ => return None
        };

        let len = self.get_rule(rule_id).len();
        let items: BTreeSet<LRItem> = (0..len).map(|idx| LRItem::new(rule_id, idx)).collect();
        Some(items)
    }


}

/// 使用LR1合并算法构建LALR
pub struct AdvancedLALR1Builder<T: SymbolBound> {
    id2item_map: IndexMap<usize, LookaheadItemSet<T>>,
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

    /// 合并LR1
    pub fn build_table(mut self) -> (IndexMap<usize, LookaheadItemSet<T>>, Vec<(usize, Symbol<T>, usize)>, usize) {
        let mut id_map = HashMap::new();
        let mut core_id_map = HashMap::new();
        let mut transitions = HashSet::new(); // 去重
        // 构建通信集表
        for (&old_id, item_set) in self.id2item_map.iter() {
            let new_id = *core_id_map
                .entry(item_set.core_set.clone())
                .or_insert_with(|| self.id_factory.next_id());
            id_map.insert(old_id, new_id);
        }
        let mut lookahead_map: IndexMap<_, _> = core_id_map.iter().map(|(item_set, &id)|
            (id, LookaheadItemSet { core_set: item_set.clone(), lookahead_map: BTreeMap::new() }))
            .collect();

        // 合并Lookahead
        for (old_id, item_set) in self.id2item_map {
            let new_id = id_map[&old_id];
            let lookahead = &mut lookahead_map.get_mut(&new_id).unwrap().lookahead_map;
            for (item, la_set) in item_set.lookahead_map { // 合并Lookahead
                lookahead.entry(item)
                    .or_insert_with(BTreeSet::new)
                    .extend(la_set);
            }

        }

        // 更新转移边
        for (from, symbol, to) in self.transition_table {
            let new_from = id_map[&from];
            let new_to = id_map[&to];
            transitions.insert((new_from, symbol, new_to));
        }


        let init_state = id_map[&self.init_state];

        (lookahead_map, transitions.into_iter().collect(), init_state)
    }




}


#[test]
fn test() {
    let rules: Vec<RuleVec<char>> = vec![
        vec![
            Rule::Expression(vec![Symbol::NonTerminal(0), Symbol::NonTerminal(0), Symbol::Terminal('a')]),
            Rule::Expression(vec![Symbol::NonTerminal(1), Symbol::Terminal('b')]),
            Rule::Expression(vec![Symbol::Terminal('c'), Symbol::NonTerminal(2)]),
            Rule::Epsilon
        ],
        vec![
            Rule::Expression(vec![Symbol::NonTerminal(0), Symbol::Terminal('d')]),
            Rule::Expression(vec![Symbol::NonTerminal(3), Symbol::Terminal('e')]),
            Rule::Expression(vec![Symbol::NonTerminal(0), Symbol::NonTerminal(0)])
        ],
        vec![
            Rule::Expression(vec![Symbol::NonTerminal(3), Symbol::Terminal('f')]),
        ],
        vec![
            Rule::Expression(vec![Symbol::Terminal('e'), Symbol::NonTerminal(3)]),
        ]
    ];

    let mut grammar = Grammar::new(0);
    for (idx, alter_rules) in rules.into_iter().enumerate() {
        grammar.add_rule(idx, alter_rules, RuleMeta::new(idx, idx.to_string()));
    }

    let builder = AdvancedLALR1Builder::new(&grammar);
    let (id2items_table,  transition, _) = builder.build_table();
    // println!("{:#?}", );

    for (from, sym, to) in transition {
        let from = id2items_table.get(&from).unwrap();
        let to  = id2items_table.get(&to).unwrap();
        let sym: &str = match sym {
            Symbol::Terminal(x) => &x.to_string(),
            Symbol::NonTerminal(x) => &grammar.get_meta(x).unwrap().name,
        };

        println!("{:?}\n\t{:?} -> {:?}", from, sym, to);
    }

    // let first = build_first(&grammar);
    // println!("{:?}", first);

    // println!("{:#?}", grammar);
    // println!("{:#?}", grammar.get_size());

    // println!("{:?}", LR0Builder::new(grammar).item_closure());

}
