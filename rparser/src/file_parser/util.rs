use std::collections::BTreeSet;
use crate::common::grammar::{EndSymbol, Grammar, ProdMeta, Rule, Symbol, SymbolMeta};
use crate::common::lr_type::{LRItem, LookaheadItemSet};

/// 将item转换回expr字符串数组
#[allow(dead_code)]
pub fn item_to_expr(item: &LRItem, grammar: &Grammar<usize>, token_map: &[SymbolMeta], prod_map: &[ProdMeta]) -> Vec<String> {
    let (rule_id, alter) = item.rule;
    let option = grammar.get_rule(rule_id).unwrap().get(alter).unwrap();

    match option {
        Rule::Epsilon => vec![],
        Rule::Expression(x) => x.iter().map(|x| match x {
            Symbol::Terminal(x) => token_map[*x].content.clone(),
            Symbol::NonTerminal(x) => prod_map[*x].name.clone(),
        }).collect()
    }
}

#[allow(dead_code)]
pub fn item_to_string(item: &LRItem, grammar: &Grammar<usize>, token_map: &[SymbolMeta], prod_map: &[ProdMeta], lookahead: &BTreeSet<EndSymbol<usize>>) -> String {
    let lookahead: Vec<_> = lookahead.iter().map(|x| match x {
        EndSymbol::End => "$$$".to_string(),
        EndSymbol::Symbol(x) => token_map[*x].content.clone(),
    }).collect();
    let mut expr = item_to_expr(item, grammar, token_map, prod_map);
    expr.insert(item.pos, ".".to_string());
    format!("[{} -> {}    {}]", prod_map[item.rule.0].name, expr.join(" "), lookahead.join(", "))
}

#[allow(dead_code)]
pub fn item_set_to_string(item_set: &LookaheadItemSet<usize>, grammar: &Grammar<usize>, token_map: &[SymbolMeta], prod_map: &[ProdMeta]) -> String {
    let items: Vec<_> = item_set.core_set.iter()
        .map(|item| item_to_string(item, grammar, token_map, prod_map, &item_set.lookahead_map[item]))
        .collect();
    items.join("\n")
}
