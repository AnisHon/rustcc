//! date: 2025/8/26
//! author: anishan
//!
//! 表格构建
//!
//! # Contents
//! - 'TableType' 构造表格类型
//! - 'LRTableBuilder' 带Lookahead文法的文法矩阵构造工具
//!

use crate::common::grammar::{Assoc, EndSymbol, Grammar, ProdMeta, Symbol, SymbolID, SymbolMeta};
use crate::common::lr_type::{ActionTable, GotoTable, LRAction};
use crate::file_parser::config_reader::{get_grammar, GrammarConfig, GrammarConfigParser, END_SYMBOL_ID};
use crate::lr::lalr1::{AdvancedLALR1Builder, LALR1Builder};
use crate::lr::lr1::LR1Builder;
use indexmap::IndexMap;
use std::cmp::Ordering;

pub enum TableType {
    LALR1,
    AdvancedLALR1,
    LR1,
}

/// 文法表格构造器
/// 文法构造器需要返回三个参数，
/// - 'item_set_map': LookaheadItemSet
/// - 'transition_table': Vec<(usize, Symbol, usize)>
/// - 'init_state': usize
///
/// # Members
/// - 'table_type': 生成表格类型
/// - 'config': ConfigReader读取的文法配置
/// - 'prod_map': 推导式信息表
/// - 'token_meta': 符号信息表
/// - 'rule_id_map': rule定位id
/// - 'grammar': 文法本身
///
///
pub struct LRTableBuilder {
    table_type: TableType,
    pub config: GrammarConfig,
    pub prod_map: Vec<ProdMeta>,
    pub token_meta: Vec<Option<SymbolMeta>>,
    rule_id_map: IndexMap<(usize, usize), usize>,
    pub grammar: Grammar<usize>,
}

impl LRTableBuilder {
    pub fn new(table_type: TableType, input: String) -> Self {
        let config = GrammarConfigParser::new(input).parse();
        let  (grammar, token_meta, prod_map) = get_grammar(&config);
        // 子式->id映射表
        let rule_id_map: IndexMap<_, _> = (0..grammar.get_size())
            .flat_map(|i| (0..grammar.get_rule(i).unwrap().len()).map(move |j| (i, j)))
            .enumerate()
            .map(|(i, j)| (j, i))
            .collect();

        Self {
            table_type,
            config,
            rule_id_map,
            prod_map,
            token_meta,
            grammar
        }
    }
    
    pub fn get_token_meta(&self, id: usize) -> &SymbolMeta {
        self.token_meta[id].as_ref().unwrap()
    }

    /// 构建LR表格
    /// # Returns
    /// `usize`: init state
    /// `GotoTable`: goto table
    /// `ActionTable`: action table
    pub fn build_lr_table(&self) -> (ActionTable, GotoTable, usize) {
        let (item_set_map, transition_table, init_state) = match self.table_type {
            TableType::LALR1 => LALR1Builder::new(&self.grammar).build_table(),
            TableType::AdvancedLALR1 => AdvancedLALR1Builder::new(&self.grammar).build_table(),
            TableType::LR1 => LR1Builder::new(&self.grammar).build_table(),
        };

        let token_sz = self.token_meta.len();
        let state_sz = item_set_map.len();


        // LALR1表 [state;token] -> [state][token] ，多出一个是结束符，默认为Error
        let mut action_table = vec![vec![LRAction::Error; token_sz + 1]; state_sz];
        // goto表，[state;ruleID] ，默认为None表示Error
        let mut goto_table = vec![vec![None; self.grammar.get_size()]; state_sz];

        // 遍历transition，设置GOTO和SHIFT
        for (from, symbol, to) in transition_table.into_iter() {
            match symbol {
                Symbol::Terminal(x) => action_table[from][x] = LRAction::Shift(to),
                Symbol::NonTerminal(x) => goto_table[from][x] = Some(to),
            }
        }
        
        // 设置Reduce和End
        for (state, item_set) in item_set_map.into_iter() {
            for item in item_set.core_set {
                if !item.is_reduced(&self.grammar) {
                    continue;
                }
                let rule_id = self.rule_id_map[&item.rule];

                // 归并项目
                let lookahead = &item_set.lookahead_map[&item];
                for symbol in lookahead {
                    let symbol_id = match symbol {
                        EndSymbol::End => END_SYMBOL_ID,
                        EndSymbol::Symbol(x) => *x
                    };

                    // 确定规约还是结束
                    let new: LRAction = match symbol {
                        EndSymbol::End if item.is_start(&self.grammar) => LRAction::Accept(rule_id), // 初始 + END + 规约 = 接受项目
                        _ => LRAction::Reduce(rule_id),
                    };

                    let origin: LRAction = action_table[state][symbol_id]; // 拷贝代价不高

                    let action = self.handle_conflict(origin, new, symbol_id); // 处理冲突

                    action_table[state][symbol_id] = action;
                }
            }

        }
        (action_table, goto_table, init_state)
    }

    ///
    /// 处理冲突，通过优先级和结核性能解决shift reduce冲突，无法解决reduce reduce冲突
    ///
    fn handle_conflict(&self, origin: LRAction, new: LRAction, symbol_id: SymbolID) -> LRAction {
        if matches!(origin, LRAction::Error) || origin == new { // 没有冲突
            return new;
        }

        // reduce-reduce冲突，无法解决，默认返回第一个
        if !new.is_shift() && !origin.is_shift() && new != origin {
            self.conflict_warning(&origin, &new, symbol_id);
            return origin
        }

        // shift-reduce冲突

        // 优先级
        let origin_priority = self.get_priority(&origin, symbol_id);
        let new_priority = self.get_priority(&new, symbol_id);

        // 结合性
        let origin_assoc = self.get_assoc(&origin, symbol_id);
        let new_assoc = self.get_assoc(&new, symbol_id);

        // 使用优先级解决冲突
        match origin_priority.cmp(&new_priority) {
            Ordering::Less => return new,
            Ordering::Greater => return origin,
            Ordering::Equal => {}
        }

        // 优先级不能解决冲突
        // 同优先级冲突
        // 使用结合性解决冲突
        // 规约移入冲突，通过结合性判断
        match (origin_assoc, new_assoc) {
            (Assoc::Right, Assoc::Right) => {  // 都是右结合，移入优先
                if origin.is_shift() { // 继续移入
                    origin
                } else {
                    new
                }
            }
            (Assoc::Left, Assoc::Left) => { // 都是左结合，规约优先
                if !origin.is_shift() {
                    origin
                } else {
                    new
                }
            },
            (Assoc::Right, Assoc::Left) | (Assoc::Left, Assoc::Right) => { // 有左有右，右结合优先，非正常行为发出警告
                self.conflict_warning(&origin, &new, symbol_id);
                if origin.is_shift() { // 继续移入
                    origin
                } else {
                    new
                }
            }
            (Assoc::NonAssoc, Assoc::NonAssoc) => { // 存在无结合性，报错停止
                self.conflict_warning(&origin, &new, symbol_id);
                panic!("failed to resolve NonAssoc Conflict");
            }
            _ => { // 其他情况规约优先，发出警告
                self.conflict_warning(&origin, &new, symbol_id);
                if origin.is_shift() { // 继续移入
                    new
                } else {
                    origin
                }
            }
        }
    }

    /// Reduce Action或者Shift Symbol的优先级
    fn get_priority(&self, action: &LRAction, symbol_id: SymbolID) -> usize {
        if action.is_shift() {
            self.get_token_meta(symbol_id).priority
        } else {
            let rule_id = action.unwrap();
            self.prod_map[rule_id].priority
        }
    }

    /// Reduce Action或者Shift Symbol的优先级
    fn get_assoc(&self, action: &LRAction, symbol_id: SymbolID) -> Assoc {
        if action.is_shift() {
            self.get_token_meta(symbol_id).assoc
        } else {
            let rule_id = action.unwrap();
            self.prod_map[rule_id].assoc
        }
    }

    /// 输出冲突警告信息
    fn conflict_warning(&self, origin: &LRAction, new: &LRAction, symbol_id: SymbolID) {
        let get_name = |action: &LRAction| {
            match action {
                LRAction::Reduce(rule_id) | LRAction::Accept(rule_id) => {
                    let meta = &self.prod_map[*rule_id];
                    format!("{}({})", meta.name, meta.alter)
                }
                LRAction::Shift(_) => {
                    self.get_token_meta(symbol_id).content.clone()
                }
                LRAction::Error => unreachable!()
            }
        };
        let origin_name = get_name(origin);
        let new_name = get_name(new);

        let error_msg = match (origin.is_shift(), new.is_shift()) {
            (false, false) => format!(
                "REDUCE-REDUCE symbol:{} -> {} {} ",
                self.get_token_meta(symbol_id).content,
                origin_name, new_name,
            ),
            (false, true) | (true, false) => format!(
                "SHIFT-REDUCE {} {}",
                origin_name, new_name,
            ),
            _ => unreachable!()
        };

        println!("Warning: Conflict: {}", error_msg);
    }
}

