use std::collections::{BTreeMap, BTreeSet, VecDeque};
use common::utils::id_util::IncIDFactory;
use common::utils::unique_id_factory::UniqueIDFactory;
use crate::common::grammar::{Grammar, SymbolBound, RuleID, SymbolVec, Symbol, RuleVec, RuleMeta, Rule};
use crate::common::lr_type::LR0Item;

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
        self.grammar.get_rule(rule_id).expect(format!("rule id {} not found", rule_id).as_str())
    }

    /// 项目集闭包
    fn item_closure(&self, items: BTreeSet<LR0Item<T>>) -> BTreeSet<LR0Item<T>> {
        let mut closure_set = BTreeSet::from_iter(items.iter().cloned()); // 集合初始化默认元素
        let mut queue: VecDeque<LR0Item<T>> =  // 初始化队列，不会压入重复元素
            VecDeque::from_iter(items.into_iter());

        while !queue.is_empty(){
            let item = queue.pop_front().unwrap();
            let next_symbol = item.next_symbol();


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
            let items = alter_rules.iter().map(LR0Item::from_rule);
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
    /// ### parameters
    /// items: 内部的LR0Item项目集，必须全是经过symbol转移的，该函数不负责过滤
    /// symbol: 下一次转移符号
    ///
    fn item_goto(&self, items: BTreeSet<&LR0Item<T>>, symbol: Symbol<T>) -> BTreeSet<LR0Item<T>> {
        let items: BTreeSet<_> = items.iter()// 移动GO操作
            .map(|item| {
                assert!(item.next_symbol().is_some()); // 非规约项目
                assert_eq!(item.next_symbol().unwrap(), symbol); // 下一个符号是当前符号
                item.move_next() // 转移
            }).collect();

        if items.is_empty() {
            BTreeSet::new()
        } else {
            self.item_closure(items)
        }
    }

    /// 获取项目集转移符号
    fn item_symbols(&self, items: &'a BTreeSet<LR0Item<T>>) -> BTreeMap<Symbol<T>, BTreeSet<&'a LR0Item<T>>> {
        let mut symbols_table = BTreeMap::new();
        for item in items.iter() {
            let symbol = match item.next_symbol() {
                None => continue,
                Some(x) => x
            };
            symbols_table.entry(symbol).or_insert(BTreeSet::new()).insert(item);
        }
        symbols_table
    }

    /// 获取初始集合
    fn init_item_set(&self) -> BTreeSet<LR0Item<T>> {
        let start_rule_id = self.grammar.get_start_rule();
        let start = self.grammar.get_rule(start_rule_id).unwrap().into_iter().map(LR0Item::from_rule);
        self.item_closure(BTreeSet::from_iter(start))
    }

    /// 构建表
    /// ### return
    /// id2items_table: id映射表items_id -> item_set
    /// lr0_table: LR0表，使用三元组表示(items_id, symbol, items_id)
    pub fn build_table(&mut self) -> (BTreeMap<usize, BTreeSet<LR0Item<T>>>, Vec<(usize, Symbol<T>, usize)>) {
        let init_set = self.init_item_set();
        let mut queue = VecDeque::from(vec![init_set]);
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
        let id2items_table = items2id_table.into_iter()
            .map(|(k, v)| (v, k))
            .collect();
        (id2items_table, lr0_table)
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
        grammar.add_rule(idx, alter_rules, RuleMeta {name: idx.to_string(), optional: false});
    }

    let init = grammar.get_rule(grammar.get_start_rule()).unwrap().into_iter().map(LR0Item::from_rule);
    let init = BTreeSet::from_iter(init);

    let mut builder = LR0Builder::new(&grammar);
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

    // let first = build_first(&grammar);
    // println!("{:?}", first);

    // println!("{:#?}", grammar);
    // println!("{:#?}", grammar.get_size());

    // println!("{:?}", LR0Builder::new(grammar).item_closure());

}

