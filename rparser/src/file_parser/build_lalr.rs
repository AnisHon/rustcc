use std::collections::HashMap;
use crate::common::grammar::{Grammar, Symbol, SymbolMeta};
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

        let token_sz = self.config.tokens.len();
        let state_sz = item_set_map.len();
        // let rule_map: HashMap<_, _> = (0..self.grammar.get_size())
        //     .map(|i| (0..self.grammar.get_rule(i).unwrap().len()).map(|j| (i, j)))
        //     .flatten()
        //     .enumerate()
        //     .map(|(i, j)| (j, i))
        //     .collect();

        // LALR1表 [state;token] -> [state][token] ，多出一个是结束符，默认为Error
        let mut action_table = vec![vec![LRAction::Error; token_sz + 1]; state_sz];
        // goto表，[state;ruleID] ，默认为None表示Error
        let mut goto_table = vec![vec![None; self.grammar.get_size()]; state_sz];

        // 遍历transition，设置GOTO和SHIFT
        for (from, symbol, to) in transition_table {
            match symbol {
                Symbol::Terminal(x) => action_table[from][x] = Shift(to),
                Symbol::NonTerminal(x) => goto_table[from][x] = Some(to),
            }
        }
        
        // 设置Reduce和End
        

        



    }



}




