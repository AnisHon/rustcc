use crate::automata_builder::dfa_builder::DFABuilder;
use crate::automata_builder::dfa_optimizer::DFAOptimizer;
use crate::automata_builder::nfa_builder::NFABuilder;
use crate::char_class::char_class_builder::CharClassBuilder;
use crate::char_class::char_class_set::CharClassSet;
use crate::common::re_err::ReResult;
use crate::lex::lex_core::re2tokens;
use crate::parser::parser_core::{to_ast, ReParser};
use crate::rlexer::lex_config::LexConfig;
use common::lex::{DFA, NFA, NFASymbol, StateID, StateMeta};
use std::collections::BTreeSet;
use crate::parser::ast::ASTNode;

pub struct Lexer {
    dfa: DFA,
    config: LexConfig,
    char_class_set: CharClassSet,
}

impl Lexer {
    pub fn new(config: LexConfig) -> Self {
        let (dfa, char_class_set) = match Self::init(&config) {
            Ok(dfa) => dfa,
            Err(e) => panic!("{}", e),
        };
        Self {
            dfa,
            config,
            char_class_set,
        }
    }

    fn init(config: &LexConfig) -> ReResult<(DFA, CharClassSet)> {
        if config.rules.is_empty() {
            panic!("No regex specified");
        }
        let rules_sz = config.rules.len();
        let builder = CharClassBuilder::new((0, 0x10FFFF)); // char_class_set 初始化为 Unicode全集
        let mut ast_nodes: Vec<ASTNode> = Vec::with_capacity(rules_sz);
        let mut ast_idx_map: Vec<usize> = Vec::with_capacity(rules_sz); // 用来映射对应的Lex结构位置

        // Regex -> Token -> CST -> AST
        // 这个场景用iter函数式不太好用
        for (idx, rule) in config.rules.iter().enumerate() {
            let regex = rule.regex.as_str();
            
            let tokens = re2tokens(regex)?; // 构建Token
            let parser = ReParser::new(tokens); // 构建Parser
            let cst_node = parser.parse().map_err(|e| e.with_re(&regex))?; // 解析得到CST
            let ast_node = to_ast(&cst_node)?; // 转换得到 AST
            ast_nodes.push(ast_node);
            ast_idx_map.push(idx);
        }

        // 构建CharClassSet
        let char_class_set = builder.build_char_class_set(&ast_nodes);

        // NFA构造器
        let mut builder = NFABuilder::new(&char_class_set);

        // 转化为NFA
        let mut nfa_vec: Vec<_> = ast_nodes
            .into_iter()
            .map(|ast_node| builder.build(&ast_node))
            .collect();

        // 设置元信息
        for (idx, nfa) in nfa_vec.iter_mut().enumerate() {
            let idx = ast_idx_map[idx];

            let terminate_states: Vec<_> = nfa.get_terminated_states().iter()
                .copied()
                .collect();

            for state_id in terminate_states {
                let meta = nfa.get_status_mut(state_id);
                let lex_rule = &config.rules[idx];
                
                meta.action = lex_rule.action.clone();
                meta.terminate = true;
                meta.id = Some(idx);
                meta.priority = Some(idx);
            }
        }

        // 合并NFA
        let nfa = nfa_vec.into_iter().reduce(|mut a, b| { //
            let b_init_state = a.merge(b);
            a.add_edge(a.get_init_state(), NFASymbol::Epsilon, b_init_state);
            a
        }).unwrap();

        // for x in nfa.get_terminated_states().iter().collect::<Vec<_>>() {
        //     println!("{:?}", nfa.get_status(*x));
        // }

        // DFA 构造
        let dfa_builder = DFABuilder::new(nfa, char_class_set.size(), Self::priority_state_nfa);
        let dfa = dfa_builder.build();



        // DFA 优化
        let optimizer = DFAOptimizer::new(dfa, Self::partition_key, Self::priority_state);

        let dfa = optimizer.optimize();

        Ok((dfa, char_class_set))
    }

    /// 节点的key，将None映射为 usize::max
    fn partition_key(meta: &StateMeta) -> usize {
        meta.id.unwrap_or(usize::MAX)
    }


    fn priority_state(dfa: &DFA, states: BTreeSet<StateID>) -> StateID {
        // 一定要注意 默认 None总是最小的，所以给None映射为最大值
        let meta = states
            .iter()
            .map(|&state| (state, dfa.get_meta(state)))
            .min_by_key(|(_, meta)| meta.priority.unwrap_or(usize::MAX));  // 查找优先级最大的（越小越大）

        meta.unwrap().0
    }


    fn priority_state_nfa(nfa: &NFA, states: &BTreeSet<StateID>) -> StateID {
        // 一定要注意 默认 None总是最小的，所以给None映射为最大值
        let meta = states
            .iter()
            .map(|&state| (state, nfa.get_status(state)))  // 转换为meta
            .min_by_key(|(_, meta)| meta.priority.unwrap_or(usize::MAX));  // 查找优先级最大的（越小越大）


        meta.unwrap().0
    }

    pub fn get_char_class_set(&self) -> &CharClassSet {
        &self.char_class_set
    }

    pub fn get_dfa(&self) -> &DFA {
        &self.dfa
    }
    
    pub fn get_config(&self) -> &LexConfig {
        &self.config
    }
}
