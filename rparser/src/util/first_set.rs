use crate::common::grammar::{EpsilonSymbol, Grammar, Rule, RuleID, RuleVec, Symbol, SymbolBound};
use std::collections::{BTreeMap, BTreeSet};
use indexmap::IndexMap;
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

/// FirstMap类型
pub type FirstMap<T> = IndexMap<RuleID, BTreeSet<EpsilonSymbol<T>>>;


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

/// 为推导式构建增量first set，并非构建完整first，只处理change=True的增量部分
fn calc_first_set<T: SymbolBound>(
    alter_rule: &RuleVec<T>,
    first_map: &FirstMap<T>,
    change_tracker: &IndexMap<RuleID, bool>
) -> BTreeSet<EpsilonSymbol<T>> {
    let mut first_set = BTreeSet::new();
    let mut nullable = false; // 全局角度能否推出空

    for rule in alter_rule.iter() {
        let expr = match rule {
            Rule::Epsilon => {
                first_set.insert(EpsilonSymbol::Epsilon);
                nullable = true; // 直接推导出Epsilon
                continue  // Epsilon什么都不用处理，跳过
            },
            Rule::Expression(x) => x
        };

        let mut expr_nullable = true; // 局部追踪能否推导空
        assert!(!expr.is_empty()); // 空集合不应该出现在这里，应该被构建为Epsilon

        for sym in expr {
            // 非终结符继续构造，合并first，确定nullable
            let next_rule_id = match sym {
                Symbol::Terminal(x) => {
                    expr_nullable = false; // 推出终结符不能推导空
                    first_set.insert(EpsilonSymbol::Symbol(x.clone()));
                    break; // 终结符停止构造
                },
                Symbol::NonTerminal(x) => x
            };

            let next_first_set = first_map.get(next_rule_id).unwrap();
            if change_tracker[next_rule_id] { // 有改变，则合并，否则跳过
                first_set.extend(next_first_set.iter().cloned());
            }

            // 不推导空，停止构造
            if !next_first_set.contains(&EpsilonSymbol::Epsilon) {
                expr_nullable = false; // 当前也一定不能推导空
                break;
            }
        }

        nullable = nullable || expr_nullable; // 表示ture优先，只要存在ture则optional为ture
    }

    if !nullable { // 不能推导空就直接删掉空
        first_set.remove(&EpsilonSymbol::Epsilon);
    }

    first_set
}

/// 计算文法规则的first集
/// ### return
/// FistMap<T>: RuleId -> First Set(BTreeSet<Symbol>)
pub fn build_first<T: SymbolBound>(grammar: &Grammar<T>) -> FirstMap<T> {
    let rules = get_rules(grammar);
    let mut first_map: FirstMap<T> = rules.iter()   // 推导式到first集合的表
        .map(|(&rule_id, _)| (rule_id, BTreeSet::new()))// 初始化表
        .collect();
    let mut change_tracker: IndexMap<RuleID, bool> = rules.iter()// first set变化表，跳过无变化项目
        .map(|(&rule_id, _)| (rule_id, false)) // 初始值为false，因为初始first是空的，可以跳过
        .collect();
    let mut changes = true; // 增量迭代法

    while changes {
        changes = false;

        for (&rule_id, alter_rules) in rules.iter() {
            let mut next_first_set = calc_first_set(alter_rules, &first_map, &change_tracker); // 计算规则的增量 first

            let first_set = first_map.get_mut(&rule_id).unwrap(); // 更新规则的first

            let prev_sz = first_set.len(); // 增量判断
            first_set.append(&mut next_first_set);
            let new_sz = first_set.len();

            change_tracker.insert(rule_id, prev_sz != new_sz); // 写两遍交给编译器局部优化
            changes = changes || prev_sz != new_sz; // ture优先
        }
    }
    first_map
}


