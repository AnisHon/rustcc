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
        let mut closure_set = BTreeSet::new();
        let mut queue: VecDeque<LR0Item<T>> =  // 构建队列，不会压入重复元素
            VecDeque::from_iter(items.into_iter());

        while !queue.is_empty(){
            let item = queue.pop_front().unwrap();
            let next_symbol = item.next_symbol();
            closure_set.insert(item);


            let symbol = match next_symbol {
                None => continue,
                Some(x) => x
            };

            let rule_id = match symbol {
                Symbol::Terminal(_) => continue,
                Symbol::NonTerminal(rule_id) => rule_id
            };

            let alter_rules = self.get_rule(rule_id);
            let items = alter_rules.iter().map(LR0Item::from_rule);
            for item in items {
                if !closure_set.contains(&item) {  // 保证不压入重复元素
                    closure_set.insert(item.clone());
                    queue.push_back(item);  // 入队
                }
            }

        }

        closure_set
    }

    /// 项目集go操作
    fn item_goto(&self, items: &BTreeSet<LR0Item<T>>, symbol: Symbol<T>) -> BTreeSet<LR0Item<T>> {
        let items: BTreeSet<_> = items.iter()// 移动GO操作
            .filter(|item| {
                match item.next_symbol() { // 过滤无关项目
                    None => false,
                    Some(x) => x == symbol
                }
            }).map(|item| {
                item.move_next() // 有关项目移动一次
            }).collect();

        if items.is_empty() {
            BTreeSet::new()
        } else {
            self.item_closure(items)
        }
    }

    /// 获取项目集转移符号
    fn item_symbols(&self, items: &BTreeSet<LR0Item<T>>) -> BTreeSet<Symbol<T>> {
        let mut symbols = BTreeSet::new();
        for item in items.iter() {
            let symbol = match item.next_symbol() {
                None => continue,
                Some(x) => x
            };
            symbols.insert(symbol);
        }

        symbols
    }

    /// 获取初始集合
    fn init_item_set(&self) -> BTreeSet<LR0Item<T>> {
        let start_rule_id = self.grammar.get_start_rule();
        let start = self.grammar.get_rule(start_rule_id).unwrap().into_iter().map(LR0Item::from_rule);
        self.item_closure(BTreeSet::from_iter(start))
    }

    /// 构建表
    /// ### return
    /// LR0Set ID table, Translation Table
    pub fn build_table(&mut self) -> (BTreeMap<BTreeSet<LR0Item<T>>, usize>, Vec<(usize, Symbol<T>, usize)>) {
        let init_set = self.init_item_set();
        let mut queue = VecDeque::from(vec![init_set]);
        let mut item_set_id_table = BTreeMap::new();
        let mut lr0_table = Vec::new();

        while !queue.is_empty() {
            let item_set = queue.pop_front().unwrap();
            let item_set_id = *item_set_id_table
                .entry(item_set.clone())
                .or_insert_with(|| self.id_factory.next_id());

            let symbols = self.item_symbols(&item_set);
            for symbol in symbols {
                let goto_set = self.item_goto(&item_set, symbol.clone());

                let goto_set_id = *item_set_id_table.entry(goto_set.clone()).or_insert_with(|| {
                    queue.push_back(goto_set.clone());
                   self.id_factory.next_id()
                });
                lr0_table.push((item_set_id, symbol ,goto_set_id));
            }
        }
        (item_set_id_table, lr0_table)
    }







}


#[test]
fn test() {
    let rules: Vec<RuleVec<char>> = vec![
        vec![
            Rule::Expression(vec![Symbol::NonTerminal(0), Symbol::NonTerminal(0), Symbol::Terminal('a')]),
            Rule::Expression(vec![Symbol::NonTerminal(1), Symbol::Terminal('b')]),
            Rule::Expression(vec![Symbol::NonTerminal(2), Symbol::Terminal('c')])
        ],
        vec![
            Rule::Expression(vec![Symbol::NonTerminal(0), Symbol::Terminal('d')]),
            Rule::Expression(vec![Symbol::NonTerminal(3), Symbol::Terminal('e')]),
        ],
        vec![
            Rule::Expression(vec![Symbol::NonTerminal(3), Symbol::Terminal('f')]),
        ],
        vec![
            Rule::Expression(vec![Symbol::NonTerminal(3), Symbol::Terminal('e')]),
        ]
    ];

    let mut grammar = Grammar::new(0);
    for (idx, alter_rules) in rules.into_iter().enumerate() {
        grammar.add_rule(idx, alter_rules, RuleMeta {name: idx.to_string(), optional: false});
    }

    let init = grammar.get_rule(grammar.get_start_rule()).unwrap().into_iter().map(LR0Item::from_rule);
    let init = BTreeSet::from_iter(init);

    let mut builder = LR0Builder::new(&grammar);
    println!("{:?}", builder.build_table());

    // println!("{:#?}", grammar);
    // println!("{:#?}", grammar.get_size());

    // println!("{:?}", LR0Builder::new(grammar).item_closure());



}

