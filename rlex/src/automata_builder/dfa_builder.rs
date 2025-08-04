use common::lex::state_id_factory::IncrementalStateIDFactory;
use common::lex::{ClassID, DFA, NFA, NFASymbol, StateID, StateMeta};
use common::utils::unique_id_factory::UniqueIDFactory;
use std::collections::{BTreeSet, HashMap, HashSet};

pub struct DFABuilder {
    nfa: NFA,
    stride: usize,
    id_factory: IncrementalStateIDFactory,
}

impl DFABuilder {
    pub fn new(nfa: NFA, stride: usize) -> Self {
        Self {
            nfa,
            stride,
            id_factory: IncrementalStateIDFactory::new(0),
        }
    }

    pub fn build(&mut self) -> DFA {
        let transaction_table = self.build_translate_table();
        let state_table = self.build_status_table(&transaction_table);
        let terminate_set = self.build_terminate_set(&state_table);
        let mut dfa = self.build_dfa(transaction_table, state_table);
        for state_id in terminate_set {
            dfa.get_meta_mut(state_id).terminate = true;
        }
        dfa
    }

    fn id(&mut self) -> StateID {
        self.id_factory.next_id()
    }

    ///
    /// NFA子集构造法： epsilon闭包
    fn epsilon_closure(&self, state_ids: &BTreeSet<StateID>) -> BTreeSet<StateID> {
        assert!(self.nfa.all_exists(state_ids));
        let mut stack: Vec<_> = state_ids.iter().copied().collect();
        let mut closure = BTreeSet::new(); // 元素少速度快，支持Hash

        while !stack.is_empty() {
            let state_id = stack.pop().unwrap();
            closure.insert(state_id); // 放入
            let states = match self.nfa.find_next(state_id, NFASymbol::Epsilon) {
                Some(next) => next,
                None => continue,
            };

            // 闭包操作
            for &state in states {
                if closure.contains(&state) {
                    continue;
                }
                stack.push(state);
            }
        }
        closure
    }

    ///
    /// NFA子集构造法： 弧转，会进行一次闭包
    fn goto(&self, state_ids: &BTreeSet<StateID>, symbol: NFASymbol) -> BTreeSet<StateID> {
        let mut closure = BTreeSet::new();

        state_ids.iter().for_each(|&state_id| {
            let states = match self.nfa.find_next(state_id, symbol) {
                None => return,
                Some(x) => x,
            };
            closure.extend(states);
        });

        self.epsilon_closure(&closure)
    }

    /// 获取所有非空转移的边
    fn get_symbols(&self, state_ids: &BTreeSet<StateID>) -> BTreeSet<NFASymbol> {
        let mut symbols = BTreeSet::new();

        for &state_id in state_ids {
            symbols.extend(self.nfa.get_symbols(state_id));
        }
        symbols.remove(&NFASymbol::Epsilon); // 除去空转移

        symbols
    }

    fn build_translate_table(&self) -> Vec<(BTreeSet<StateID>, ClassID, BTreeSet<StateID>)> {
        let mut stack = vec![self.epsilon_closure(&BTreeSet::from([self.nfa.get_init_state()]))];
        let mut table = Vec::new(); // 数量多，元素复杂，无需查询
        let mut visited = HashSet::new();

        while !stack.is_empty() {
            let states = stack.pop().unwrap();
            let symbols = self.get_symbols(&states);

            for sym in symbols {
                let goto_states = self.goto(&states, sym);
                let sym = match sym {
                    NFASymbol::Epsilon => panic!("Got Epsilon, That Shouldn't happen"),
                    NFASymbol::ClassID(x) => x,
                };
                table.push((states.clone(), sym, goto_states.clone()));

                if !visited.contains(&goto_states) {
                    visited.insert(goto_states.clone());
                    stack.push(goto_states);
                };
            }
        }
        table
    }

    fn build_status_table(
        &mut self,
        translate_table: &Vec<(BTreeSet<StateID>, ClassID, BTreeSet<StateID>)>,
    ) -> HashMap<BTreeSet<StateID>, StateID> {
        let mut table = HashMap::new();

        translate_table.iter().for_each(|(states, _, goto_states)| {
            table.entry(states.clone()).or_insert_with(|| self.id());
            table
                .entry(goto_states.clone())
                .or_insert_with(|| self.id());
        });

        table
    }

    fn build_dfa(
        &self,
        translate_table: Vec<(BTreeSet<StateID>, ClassID, BTreeSet<StateID>)>,
        state_table: HashMap<BTreeSet<StateID>, StateID>,
    ) -> DFA {
        let init_state = self.epsilon_closure(&BTreeSet::from([self.nfa.get_init_state()]));
        let init_state = state_table[&init_state];
        let mut dfa = DFA::new(init_state, state_table.len(), self.stride);

        state_table.iter().for_each(|(_, &state_id)| {
            dfa.add_state(state_id, StateMeta::default());
        });

        translate_table
            .into_iter()
            .for_each(|(states, symbol, goto_states)| {
                dfa.add_transition((state_table[&states], symbol, state_table[&goto_states]));
            });

        dfa
    }

    fn build_terminate_set(
        &self,
        state_table: &HashMap<BTreeSet<StateID>, StateID>,
    ) -> BTreeSet<StateID> {
        let mut terminate_set = BTreeSet::new();
        let terminate_states = self.nfa.get_terminated_states();

        state_table.iter().for_each(|(nfa_state_ids, &state)| {
            for nfa_state_id in terminate_states {
                if nfa_state_ids.contains(&nfa_state_id) {
                    terminate_set.insert(state);
                    return;
                }
            }
        });

        terminate_set
    }
}
