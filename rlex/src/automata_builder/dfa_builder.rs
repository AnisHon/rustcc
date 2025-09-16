use common::lex::state_id_factory::IncrementalStateIDFactory;
use common::lex::{ClassID, NFASymbol, StateID, DFA, NFA};
use common::utils::unique_id_factory::UniqueIDFactory;
use indexmap::IndexMap;
use std::collections::{BTreeSet, HashSet};

///
/// DFA构造器
/// 
/// # Members
/// 
/// `nfa`: NFA
/// `stride`: DFA矩阵宽度，一般都是CharClassSet的大小
/// `id_factory`: ID生成器
/// `priority_status`: 冲突处理器，当终结节点发生冲突时调用
/// 
pub struct DFABuilder {
    nfa: NFA,
    stride: usize,
    id_factory: IncrementalStateIDFactory,
    priority_status: fn(&NFA, &BTreeSet<StateID>) -> StateID,
}

impl DFABuilder {
    pub fn new(
        nfa: NFA,
        stride: usize,
        priority_status: fn(&NFA, &BTreeSet<StateID>) -> StateID,
    ) -> Self {
        Self {
            nfa,
            stride,
            id_factory: IncrementalStateIDFactory::new(0),
            priority_status,
        }
    }

    pub fn build(mut self) -> DFA {
        let transaction_table = self.build_translate_table();
        let state_table = self.build_status_table(&transaction_table);
        // let terminate_set = self.build_terminate_set(&state_table);
        self.build_dfa(transaction_table, state_table)
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

        while  let Some(state_id) = stack.pop() {
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

        while let Some(states) = stack.pop() {
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
        translate_table: &[(BTreeSet<StateID>, ClassID, BTreeSet<StateID>)],
    ) -> IndexMap<BTreeSet<StateID>, StateID> {
        let mut table = IndexMap::new();

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
        state_table: IndexMap<BTreeSet<StateID>, StateID>,
    ) -> DFA {
        let init_state = self.epsilon_closure(&BTreeSet::from([self.nfa.get_init_state()]));
        let init_state = state_table[&init_state];
        let mut dfa = DFA::new(init_state, state_table.len(), self.stride);

        state_table.iter().for_each(|(state_set, &state_id)| {
            if state_set.is_empty() {
                return;
            }
            let state = (self.priority_status)(&self.nfa, state_set);
            dfa.add_state(state_id, self.nfa.get_status(state).clone());
        });

        translate_table
            .into_iter()
            .for_each(|(states, symbol, goto_states)| {
                dfa.add_transition((state_table[&states], symbol, state_table[&goto_states]));
            });

        dfa
    }
}
