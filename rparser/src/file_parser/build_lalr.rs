use indexmap::IndexMap;
use crate::common::grammar::{EndSymbol, Grammar, Symbol, SymbolMeta};
use crate::common::lr_type::LRAction;
use crate::common::lr_type::LRAction::Shift;
use crate::file_parser::reader::{get_grammar, GrammarConfig, GrammarConfigParser};
use crate::lr::lalr1::{AdvancedLALR1Builder, LALR1Builder};

pub struct LALR1Reader {
    advanced: bool,
    config: GrammarConfig,
    token_meta: Vec<SymbolMeta>,
    pub grammar: Grammar<usize>,
}

impl LALR1Reader {
    fn new(advanced: bool, input: String) -> Self {
        let config = GrammarConfigParser::new(input).parse();
        let  (grammar, token_meta) = get_grammar(&config);
        Self {
            advanced,
            config,
            token_meta,
            grammar
        }
    }

    fn build_lalr1_table(&mut self) {
        let (item_set_map, transition_table, init_state) = match self.advanced {
            true => AdvancedLALR1Builder::new(&self.grammar).build_table(),
            false => LALR1Builder::new(&self.grammar).build_table()
        };

        let end_symbol_id = self.config.tokens.len(); // 始终占用最后一个ID

        let rule_map: IndexMap<_, _> = (0..self.grammar.get_size())
            .map(|i| (0..self.grammar.get_rule(i).unwrap().len()).map(move |j| (i, j)))
            .flatten()
            .enumerate()
            .map(|(i, j)| (j, i))
            .collect();

        let token_sz = self.config.tokens.len();
        let state_sz = item_set_map.len();

        // LALR1表 [state;token] -> [state][token] ，多出一个是结束符，默认为Error
        let mut action_table = vec![vec![LRAction::Error; token_sz + 1]; state_sz];
        // goto表，[state;ruleID] ，默认为None表示Error
        let mut goto_table = vec![vec![None; self.grammar.get_size()]; state_sz];

        // 遍历transition，设置GOTO和SHIFT
        for (from, symbol, to) in transition_table.into_iter() {
            match symbol {
                Symbol::Terminal(x) => action_table[from][x] = Shift(to),
                Symbol::NonTerminal(x) => goto_table[from][x] = Some(to),
            }
        }
        
        // 设置Reduce和End
        for (_, item_set) in item_set_map.into_iter() {
            for item in item_set.core_set {
                if !item.is_reduced(&self.grammar) {
                    continue;
                }
                let rule_id = rule_map[item.pos];

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
                    let origin: LRAction = action_table[rule_id][symbol_id].clone(); // 代价不高

                    let action = self.handle_conflict(origin, new); // 处理冲突

                    action_table[rule_id][symbol_id] = action;
                }
            }

        }
    }

    /// 处理冲突
    fn handle_conflict(&self, origin: LRAction, new: LRAction) -> LRAction {
        if matches!(origin, LRAction::Error) { // 未设置直接覆盖
            return new;
        }


        // 表示是否为规约项目
        let reduce_conflict1 = is_reduce(&new);
        let reduce_conflict2 = is_reduce(&origin);

        if reduce_conflict1 && reduce_conflict2 {  // 规约规约
            let rule_id1 = reduce_rule_id(&new);
            let rule_id2 = reduce_rule_id(&origin);


        } else { // 规约移入
            let (rule_id, token) = if reduce_conflict1 {
                (reduce_rule_id(&new), token_id(&origin))
            } else {
                (reduce_rule_id(&origin), token_id(&new))
            };





        }

        new
    }

}

fn is_reduce(action: &LRAction) -> bool {
    match action {
        LRAction::Reduce(_) => true,
        LRAction::End(_) => true,
        _ => false
    }
}

fn reduce_rule_id(action: &LRAction) -> usize {
    *match action {
        LRAction::Reduce(x) => x,
        LRAction::End(x) => x,
        _ => panic!("Not Reduce")
    }
}

fn token_id(action: &LRAction) -> usize {
    match action {
        Shift(x) => *x,
        _ => panic!("Not Shift")
    }
}

