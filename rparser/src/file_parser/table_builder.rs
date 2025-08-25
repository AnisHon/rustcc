use std::io::BufRead;
use indexmap::IndexMap;
use crate::common::grammar::{Assoc, EndSymbol, Grammar, Symbol, SymbolID, SymbolMeta};
use crate::common::lr_type::{LRAction};
use crate::file_parser::config_reader::{get_grammar, GrammarConfig, GrammarConfigParser};
use crate::lr::lalr1::{AdvancedLALR1Builder, LALR1Builder};
use crate::lr::lr1::LR1Builder;

pub enum TableType {
    LALR1,
    AdvancedLALR1,
    LR1,
}

pub struct LRTableBuilder {
    table_type: TableType,
    pub config: GrammarConfig,
    pub token_meta: Vec<SymbolMeta>,
    pub rule_map: Vec<(usize, usize)>,
    rule_id_map: IndexMap<(usize, usize), usize>,
    pub grammar: Grammar<usize>,
}

impl LRTableBuilder {
    pub fn new(table_type: TableType, input: String, lex_buf: impl BufRead) -> Self {
        let config = GrammarConfigParser::new(input).parse();
        let  (grammar, token_meta) = get_grammar(&config, lex_buf);
        // 子式->id映射表
        let rule_id_map: IndexMap<_, _> = (0..grammar.get_size())
            .map(|i| (0..grammar.get_rule(i).unwrap().len()).map(move |j| (i, j)))
            .flatten()
            .enumerate()
            .map(|(i, j)| (j, i))
            .collect();
        // id->子式映射表
        let rule_map: Vec<_> = rule_id_map.iter()
            .map(|(&pos, _)| pos).collect();

        Self {
            table_type,
            config,
            rule_map,
            rule_id_map,
            token_meta,
            grammar
        }
    }

    /// 构建LR表格
    /// ### return
    /// Vec<Vec<LRAction>>: action table
    /// Vec<Vec<Option<usize>>>: goto table
    /// usize: init state
    ///
    pub fn build_lr_table(&self) -> (Vec<Vec<LRAction>>, Vec<Vec<Option<usize>>>, usize) {
        let (item_set_map, transition_table, init_state) = match self.table_type {
            TableType::LALR1 => LALR1Builder::new(&self.grammar).build_table(),
            TableType::AdvancedLALR1 => AdvancedLALR1Builder::new(&self.grammar).build_table(),
            TableType::LR1 => LR1Builder::new(&self.grammar).build_table(),
        };

        let end_symbol_id = self.config.tokens.len(); // 始终占用最后一个ID




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
                let rule_id = self.rule_id_map[item.pos];

                // 归并项目
                let lookahead = &item_set.lookahead_map[&item];
                for symbol in lookahead {
                    let symbol_id = match symbol {
                        EndSymbol::End => end_symbol_id,
                        EndSymbol::Symbol(x) => *x
                    };

                    let new: LRAction = match symbol {
                        EndSymbol::End => LRAction::End(rule_id),
                        EndSymbol::Symbol(_) => LRAction::Reduce(rule_id)
                    };
                    let origin: LRAction = action_table[state][symbol_id]; // 拷贝代价不高

                    let action = self.handle_conflict(origin, new, symbol_id); // 处理冲突

                    action_table[state][symbol_id] = action;
                }
            }

        }
        (action_table, goto_table, init_state)
    }

    /// 处理冲突
    fn handle_conflict(&self, origin: LRAction, new: LRAction, symbol_id: SymbolID) -> LRAction {
        if matches!(origin, LRAction::Error) { // 没有冲突
            return new;
        }

        // reduce-reduce冲突，无法解决，默认返回第一个
        if !new.is_shift() && !origin.is_shift() {
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
        if origin_priority < new_priority {
            return new
        } else if origin_priority > new_priority {
            return origin
        }
        /*
         优先级不能解决冲突
         同优先级冲突
         使用结合性解决冲突
         */
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
            _ => { // 存在无结合性，报错停止
                self.conflict_warning(&origin, &new, symbol_id);
                panic!("failed to resolve NoAssoc Conflict");
            }
        }
    }

    fn get_priority(&self, action: &LRAction, symbol_id: SymbolID) -> usize {
        let x = *match action {
            LRAction::Reduce(x) => x,
            LRAction::End(x) => x,
            LRAction::Shift(x) => x,
            _ => unreachable!()
        };

        if action.is_shift() {
            self.token_meta[symbol_id].priority
        } else {
            let (id, idx) = self.rule_map[x];
            self.grammar.get_meta(id).unwrap().priority[idx]
        }
    }

    fn get_assoc(&self, action: &LRAction, symbol_id: SymbolID) -> Assoc {
        let x = *match action {
            LRAction::Reduce(x) => x,
            LRAction::End(x) => x,
            LRAction::Shift(x) => x,
            _ => unreachable!()
        };
        if action.is_shift() {
            self.token_meta[symbol_id].assoc
        } else {
            let (id, idx) = self.rule_map[x];
            self.grammar.get_meta(id).unwrap().assoc[idx]
        }
    }

    fn conflict_warning(&self, origin: &LRAction, new: &LRAction, symbol_id: SymbolID) {
        let get_name = |action: &LRAction| {
            match action {
                LRAction::Reduce(x) | LRAction::End(x) => {
                    let (id, _) = self.rule_map[*x];
                    self.grammar.get_meta(id).unwrap().name.as_str()
                }
                LRAction::Shift(_) => {
                    self.token_meta[symbol_id].content.as_str()
                }
                LRAction::Error => unreachable!()
            }
        };
        let origin_name = get_name(origin);
        let new_name = get_name(new);

        let error_type = match (origin.is_shift(), new.is_shift()) {
            (false, false) => "REDUCE-REDUCE",
            (false, true) | (true, false) => "SHIFT-REDUCE",
            _ => unreachable!()
        };

        println!("Warning: {} Conflict: {} {}", error_type, origin_name, new_name);
    }
}




