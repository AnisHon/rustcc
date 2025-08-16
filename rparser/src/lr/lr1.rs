use crate::common::grammar::{EndSymbol, EpsilonSymbol, Grammar, Rule, RuleID, RuleMeta, RuleVec, Symbol, SymbolBound};
use crate::common::lr_type::LRItem;
use crate::util::first_set::build_first;
use common::utils::id_util::IncIDFactory;
use common::utils::unique_id_factory::UniqueIDFactory;
use petgraph::data::Build;
use petgraph::dot::Dot;
use petgraph::visit::NodeRef;
use petgraph::Graph;
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use crate::util::set_utils;

#[derive(Debug, Clone)]
#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct LR1ItemSet<T: SymbolBound> {
    item_set: BTreeSet<LRItem>,
    lookahead_map: BTreeMap<LRItem, BTreeSet<EndSymbol<T>>>
}

/// 只构建DFA状态机不检查冲突
pub struct LR1Builder<'a, T: SymbolBound> {
    grammar: &'a Grammar<T>,
    id_factory: IncIDFactory,
    first_map: BTreeMap<RuleID, BTreeSet<EpsilonSymbol<T>>>
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
        self.grammar.get_rule(rule_id).expect(format!("rule id {} not found", rule_id).as_str())
    }

    /// 跳过next_symbol，计算 first_set \[A ->x·BCx, xx\] FIRST(Cx xx) 跳过了B计算Cx而是BCx
    fn calc_lookahead(&self, item: &LRItem, lookahead: &BTreeSet<EndSymbol<T>>) -> BTreeSet<EndSymbol<T>> {
        let mut lookahead_first_set: BTreeSet<EndSymbol<T>> = BTreeSet::new(); // 计算Lookahead是就是计算first 

        let (rule_id, alter_idx) = item.rule;
        let expr = self.get_expr(rule_id, alter_idx).unwrap_expr();
        
        let mut nullable = true; 
        for idx in ((item.pos + 1)..expr.len()) {
            let rule_id = match &expr[idx] {
                Symbol::Terminal(x) => { // 终结符不能推出空，非nullable，停止迭代
                    lookahead_first_set.insert(EndSymbol::Symbol(x.clone()));
                    nullable = false;
                    break
                },
                Symbol::NonTerminal(x) => x
            };

            let first_set = &self.first_map[rule_id];
            lookahead_first_set.extend(
                first_set.iter() 
                    .filter(|&x| x.ne(&EpsilonSymbol::Epsilon))
                    .map(|x| EndSymbol::Symbol(x.unwrap_symbol()))
            );
            
            if !first_set.contains(&EpsilonSymbol::Epsilon) { // 不能推出空退出
                nullable = false;
                break;
            }
        }
        
        if nullable {  // 全部推出空则加入item集
            lookahead_first_set.extend(lookahead.iter().cloned());
        }

        lookahead_first_set
    }
    

    /// 项目集闭包
    fn item_closure(&mut self, item_set: &mut LR1ItemSet<T>) {
        let lookahead_map = &mut item_set.lookahead_map;
        let closure_set = &mut item_set.item_set; // 集合初始化默认元素
        let mut queue: VecDeque<LRItem> = VecDeque::from_iter(closure_set.iter().cloned()); // 初始化队列，不会压入重复元素

        while !queue.is_empty(){
            let item = queue.pop_front().unwrap();
            let next_symbol = item.next_symbol(self.grammar);
            let lookahead = lookahead_map
                .entry(item.clone())
                .or_insert_with(BTreeSet::new);

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
                    .or_insert_with(BTreeSet::new);

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
    fn item_goto(&mut self, item_set: LR1ItemSet<T>, symbol: Symbol<T>) -> LR1ItemSet<T> {
        let mut next_item_set = LR1ItemSet {item_set: BTreeSet::new(), lookahead_map: BTreeMap::new() };
        next_item_set.item_set.extend(item_set.item_set.into_iter().map(|item| {
            assert!(item.next_symbol(self.grammar).is_some()); // 非规约项目
            assert_eq!(item.next_symbol(self.grammar).unwrap(), symbol); // 下一个符号是当前符号
            let lookahead = item_set.lookahead_map.get(&item).unwrap().clone();
            let next_item = item.move_next(self.grammar);
            next_item_set.lookahead_map.insert(next_item.clone(), lookahead); // 继承lookahead
            next_item
        }));

        self.item_closure(&mut next_item_set);
        next_item_set
    }

    /// 获取项目集转移符号
    fn item_symbols(&self, item_set: &'a LR1ItemSet<T>) -> BTreeMap<Symbol<T>, LR1ItemSet<T>> {
        let mut symbols_table = BTreeMap::new();
        for item in item_set.item_set.iter() {
            let symbol = match item.next_symbol(self.grammar) {
                None => continue,
                Some(x) => x
            };
            let next_item_set = symbols_table.entry(symbol)
                .or_insert_with(|| LR1ItemSet { item_set: BTreeSet::new(), lookahead_map: BTreeMap::new() });
            next_item_set.item_set.insert(item.clone());
            next_item_set.lookahead_map.insert(item.clone(), item_set.lookahead_map[item].clone());
        }
        symbols_table
    }

    /// 获取初始集合
    fn init_item_set(&mut self) -> LR1ItemSet<T> {
        let start_rule_id = self.grammar.get_start_rule();
        let alter_sz = self.grammar.get_rule(start_rule_id).unwrap().len();

        // 初始化项目集
        let items: BTreeSet<_> = (0..alter_sz).map(|idx| LRItem::new(start_rule_id, idx)).collect();
        let lookahead_map = items.iter().cloned()// 初始化结束符
            .map(|item| (item, BTreeSet::from([EndSymbol::End]))).collect();
        let mut item_set = LR1ItemSet {
            item_set: items,
            lookahead_map
        };
        self.item_closure(&mut item_set);
        item_set
    }

    pub fn build_table(&mut self) -> (BTreeMap<usize, LR1ItemSet<T>>, Vec<(usize, Symbol<T>, usize)>) {
        let init_set = self.init_item_set();
        let mut queue = VecDeque::from(vec![init_set]);
        let mut items2id_table = BTreeMap::new(); // item_set -> item_id
        let mut lr1_table = Vec::new();

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
                lr1_table.push((items_id, symbol, goto_set_id));
            }
        }

        let id2items_table = items2id_table.into_iter()
            .map(|(k, v)| (v, k))
            .collect();
        (id2items_table, lr1_table)
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
        let name = char::from_u32((idx + 'A' as usize) as u32).unwrap().to_string();
        grammar.add_rule(idx, alter_rules, RuleMeta { name });
    }

    let mut builder = LR1Builder::new(&grammar);
    let (id2items_table,  transition) = builder.build_table();
    // println!("{:#?}", );

    // for (from, sym, to) in transition {
    //     let from = id2items_table.get(&from).unwrap();
    //     let to  = id2items_table.get(&to).unwrap();
    //     let sym: &str = match sym {
    //         Symbol::Terminal(x) => &x.to_string(),
    //         Symbol::NonTerminal(x) => &grammar.get_meta(x).unwrap().name,
    //     };
    //
    //     println!("{:?}\n\t{:?} -> {:?}", from, sym, to);
    // }

    let mut graph = Graph::<String, String>::new();
    let mut node_map = HashMap::new();

    for (id, item_set) in id2items_table {
        let mut format_str = String::new();

        for item in item_set.item_set.iter() {
            format_str.push('[');
            let (rule_id, idx) = item.rule;
            let name = &grammar.get_meta(rule_id).unwrap().name;

            let rule = match &grammar.get_rule(rule_id).unwrap()[idx] {
                Rule::Epsilon => "ε".to_string(),
                Rule::Expression(x) =>
                x.iter().map(|sym| match sym {
                    Symbol::Terminal(x) => x.to_string(),
                    Symbol::NonTerminal(id) => grammar.get_meta(*id).unwrap().name.clone(),
                }).collect()
            };

            format_str.push_str(name);
            format_str.push_str(" -> ");
            format_str.push_str(&rule[0..item.pos]);
            format_str.push('·');
            if item.pos < rule.len() {
                format_str.push_str(&rule[item.pos..]);
            }

            let lookahead = &item_set.lookahead_map[item];
            format_str.push_str(", ");

            for x in lookahead {
                let symbol = match x {
                    EndSymbol::End => "$$$".to_string(),
                    EndSymbol::Symbol(x) => x.to_string()
                };
                format_str.push_str(symbol.as_str());
                format_str.push(' ')
            }

            format_str.push_str("]\n");
        }

        // println!("{}", format_str);
        let index = graph.add_node(format_str);
        node_map.insert(id, index);
    }

    for (from, symbol,  to ) in transition {
        let symbol = match symbol {
            Symbol::Terminal(x) => x.to_string(),
            Symbol::NonTerminal(id) => grammar.get_meta(id).unwrap().name.clone()
        };

        graph.add_edge(node_map[&from], node_map[&to], symbol);
    }


    println!(
        "{}",
        Dot::with_attr_getters(
            &graph,
            &[],
            &|_graph, edge| {
                let node = edge.weight();
                // 节点属性
                let label = node.replace("\n", "\\n"); // Graphviz 识别换行
                format!("shape=box,label=\"{}\"", label)
            },
            &|_graph, (_idx, edge)| {
                // 边属性
                format!("shape=box,label=\"{}\"", edge)

            }
        )
    );
}


