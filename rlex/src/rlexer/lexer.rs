use crate::automata_builder::dfa_builder::DFABuilder;
use crate::automata_builder::dfa_optimizer::DFAOptimizer;
use crate::automata_builder::nfa_builder::NFABuilder;
use crate::char_class::char_class_builder::CharClassBuilder;
use crate::char_class::char_class_set::CharClassSet;
use crate::common::re_err::ReResult;
use crate::lex::lex_core::re2tokens;
use crate::parser::parser_core::ReParser;
use crate::rlexer::lex_config::LexStruct;
use common::lex::{DFA, NFA, NFASymbol, StateID, StateMeta};
use std::collections::BTreeSet;

pub struct Lexer {
    dfa: DFA,
    lex: Vec<LexStruct>,
    char_class_set: CharClassSet,
}

impl Lexer {
    pub fn new(lex: Vec<LexStruct>) -> Self {
        let (dfa, char_class_set) = match Self::init(&lex) {
            Ok(dfa) => dfa,
            Err(e) => panic!("{}", e),
        };
        Self {
            dfa,
            lex,
            char_class_set,
        }
    }

    fn init(lex: &[LexStruct]) -> ReResult<(DFA, CharClassSet)> {

        if lex.is_empty() {
            panic!("No regex specified");
        }
        let builder = CharClassBuilder::new((0, 0x10FFFF));
        let mut parsers = Vec::new();

        for lex_struct in lex.iter() {
            if lex_struct.skip { // 跳过占位项目
                continue;
            }
            let tokens = re2tokens(&lex_struct.regex)?;
            let parser = ReParser::new(tokens).map_err(|e| e.with_re(&lex_struct.regex))?;
            parsers.push(parser);
        }

        let char_class_set =
            builder.build_char_class_set(parsers.iter().map(|parser| parser.get_ast()).collect());

        let mut builder = NFABuilder::new(char_class_set.clone());

        let nfa = parsers
            .into_iter()
            .enumerate()
            .map(|(idx, parser)| {
                let mut nfa = builder.build(parser.get_ast());
                for state_id in nfa
                    .get_terminated_states()
                    .iter()
                    .copied()
                    .collect::<Vec<_>>()
                {
                    let meta = nfa.get_status_mut(state_id);
                    meta.name = Some(lex[idx].name.clone());
                    meta.terminate = true;
                    meta.id = idx;
                    meta.priority = idx;
                }

                nfa
            })
            .reduce(|mut a, b| {
                let b_init_state = a.merge(b);
                a.add_edge(a.get_init_state(), NFASymbol::Epsilon, b_init_state);
                a
            })
            .unwrap();

        let dfa_builder = DFABuilder::new(nfa, char_class_set.size(), Self::priority_state_nfa);
        let dfa = dfa_builder.build();
        let optimizer = DFAOptimizer::new(dfa, Self::partition_key, Self::priority_state);

        let dfa = optimizer.optimize();

        Ok((dfa, char_class_set))
    }

    fn partition_key(meta: &StateMeta) -> usize {
        meta.id
    }

    fn priority_state(dfa: &DFA, states: BTreeSet<StateID>) -> StateID {
        let mut vec: Vec<_> = states
            .iter()
            .map(|&state| (state, dfa.get_meta(state)))
            .collect();

        vec.sort_by_key(|(_, meta)| meta.priority);
        vec.first().unwrap().0
    }

    fn priority_state_nfa(nfa: &NFA, states: &BTreeSet<StateID>) -> StateID {
        let mut vec: Vec<_> = states
            .iter()
            .map(|&state| (state, nfa.get_status(state)))
            .collect();

        vec.sort_by_key(|(_, meta)| meta.priority);
        vec.first().unwrap().0
    }

    pub fn get_lex(&self) -> &Vec<LexStruct> {
        &self.lex
    }

    pub fn get_char_class_set(&self) -> &CharClassSet {
        &self.char_class_set
    }

    pub fn get_dfa(&self) -> &DFA {
        &self.dfa
    }
}
