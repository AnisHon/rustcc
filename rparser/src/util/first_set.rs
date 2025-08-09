use std::collections::{BTreeMap, BTreeSet, HashMap};
use crate::common::grammar::{Grammar, SymbolBound, Rule, RuleID, RuleVec, Symbol, EpsilonSymbol};
//
// struct FirstSetBuilder<T> {
//
// }
//
// impl<T> FirstSetBuilder<T> {
//
//
//     fn build(grammar: Grammar<T>)
//
// }
pub type FirstMap<T> = BTreeMap<RuleID, BTreeSet<EpsilonSymbol<T>>>;
pub fn is_optional<T: SymbolBound>(rule_vec: &RuleVec<T>) -> bool {
    for rule in rule_vec {
        match rule {
            Rule::Epsilon => return true,
            Rule::Expression(_) => continue
        }
    }
    false
}


pub fn get_rules<T: SymbolBound>(grammar: &Grammar<T>) -> BTreeMap<usize, &RuleVec<T>> {
    let mut rules = BTreeMap::new(); // 数量较少，缓存友好

    for rule_id in 0..grammar.get_size() {
        let rule = match grammar.get_rule(rule_id) {
            None => continue,
            Some(x) => x,
        };
        rules.insert(rule_id, rule);
    }

    rules
}

fn compute_first_set<T: SymbolBound>(alter_rule: &RuleVec<T>, first_map: &FirstMap<T>) -> BTreeSet<EpsilonSymbol<T>> {
    let mut first_set = BTreeSet::new();
    let mut optional = false; // 全局角度能否推出空

    for rule in alter_rule.iter() {
        let expr = match rule {
            Rule::Epsilon => {
                first_set.insert(EpsilonSymbol::Epsilon);
                continue
            },
            Rule::Expression(x) => x
        };

        let mut is_epsilon = true; // 局部追踪能否推导空
        assert!(!expr.is_empty()); // 空集合不应该出现在这里，应该被构建为Epsilon

        for sym in expr {
            let next_rule_id = match sym {
                Symbol::Terminal(x) => {
                    is_epsilon = false; // 推出终结符不能推导空
                    first_set.insert(EpsilonSymbol::Symbol(x.clone()));
                    break; // 终结符停止构造
                },
                Symbol::NonTerminal(x) => x
            };

            let next_first_set = first_map.get(&next_rule_id).unwrap();
            first_set.extend(next_first_set.iter().cloned());

            if !next_first_set.contains(&EpsilonSymbol::Epsilon) { // 停止构造
                is_epsilon = false; // 推出不推导空的非终结符不能推导空
                break;
            }
        }

        optional = optional || is_epsilon; // 表示ture优先，只要存在ture则optional为ture
    }

    if !optional { // 不能推导空直接删掉空
        first_set.remove(&EpsilonSymbol::Epsilon);
    }

    first_set
}
pub fn build_first_set<T: SymbolBound>(grammar: &Grammar<T>) {
    let rules = get_rules(grammar);
    let mut first_map: FirstMap<T> = BTreeMap::new(); // 推导式到first集合的表

    for (&rule_id, _) in rules.iter() {
        first_map.insert(rule_id, BTreeSet::new()); // 初始化一下
    }


    let mut changes = true; // 增量迭代法

    while changes {
        changes = false;

        for (rule_id, alter_rules) in rules.iter() {
            let mut next_first_set = compute_first_set(alter_rules, &first_map); // 计算规则的first

            let first_set = first_map.get_mut(&rule_id).unwrap(); // 更新规则的first

            let before = first_set.len(); // 增量判断
            first_set.append(&mut next_first_set);
            let after = first_set.len();

            changes = changes || before != after; // ture优先

        }
    }

}


