use common::lex::state_id_factory::IncrementalStateIDFactory;
use common::lex::{ClassID, DFA, StateID, StateMeta};
use common::utils::unique_id_factory::UniqueIDFactory;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

/// DFA优化，惰性求值
///
/// ### 成员
/// dfa: 原DFA
/// states: 原DFA的所有状态
/// symbol_tables: DFA state -> {symbols}的映射
/// partition_key: 初始划分的key回调
/// priority_status: 冲突情况下的确定优先级回调
/// id_factory: 自增ID Factory
///
pub struct DFAOptimizer {
    dfa: DFA,
    states: Vec<StateID>,
    symbol_table: Vec<BTreeSet<ClassID>>,
    reverse_table: HashMap<(StateID, ClassID), BTreeSet<StateID>>,

    partition_key: fn(&StateMeta) -> usize,
    priority_status: fn(&DFA, BTreeSet<StateID>) -> StateID,
    id_factory: IncrementalStateIDFactory,
}

impl DFAOptimizer {
    pub fn new(
        dfa: DFA,
        partition_key: fn(&StateMeta) -> usize,
        priority_status: fn(&DFA, BTreeSet<StateID>) -> StateID,
    ) -> Self {
        let states = Self::get_states(&dfa);
        let symbol_table = Self::symbol_table(&dfa, &states);
        let reverse_table = Self::reverse_dfa_table(&dfa, &symbol_table);
        Self {
            dfa,
            states,
            symbol_table,
            reverse_table,
            partition_key,
            priority_status,
            id_factory: IncrementalStateIDFactory::new(0),
        }
    }

    /// 优化DFA
    pub fn optimize(mut self) -> DFA {
        let partition_set = self.hopcroft();
        let partition_table = self.build_partition_table(partition_set);
        let transaction_table = self.build_transaction_table(&partition_table);
        self.build_dfa(partition_table, transaction_table)
    }

    /// 获取所有存在状态
    fn get_states(dfa: &DFA) -> Vec<StateID> {
        (0..dfa.size())
            .into_iter()
            .filter(|&state| dfa.is_exist(state))
            .collect()
    }

    /// 翻转DFA(to, symbol) -> {from}，由于太少了所以BTreeSet更高效
    fn reverse_dfa_table(
        dfa: &DFA,
        symbol_table: &Vec<BTreeSet<StateID>>,
    ) -> HashMap<(StateID, ClassID), BTreeSet<StateID>> {
        let mut reverse_table = HashMap::new();
        reverse_table.reserve(dfa.size());

        for (from, symbols) in symbol_table.iter().enumerate() {
            for &symbol in symbols {
                let to = match dfa.find_next(from, symbol) {
                    Some(to) => to,
                    None => panic!("Bug!!"),
                };

                // 填充表格 (from, symbol, to) -> (to, symbol, from)
                reverse_table
                    .entry((to, symbol))
                    .or_insert_with(|| BTreeSet::new())
                    .insert(from);
            }
        }

        reverse_table
    }

    /// 符号表 State -> {Symbols}
    fn symbol_table(dfa: &DFA, states: &Vec<StateID>) -> Vec<BTreeSet<ClassID>> {
        let mut symbol_table = Vec::new();
        symbol_table.reserve(dfa.size());

        for &state_id in states.iter() {
            let set = dfa.get_symbols(state_id).into_iter().collect();
            symbol_table.push(set);
        }

        symbol_table
    }

    fn next_id(&mut self) -> StateID {
        self.id_factory.next_id()
    }

    /// 初始划分
    fn init_partition(&self) -> Vec<BTreeSet<StateID>> {
        let mut partition: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();

        // 根据 key 分组
        for &state_id in self.states.iter() {
            let meta = self.dfa.get_meta(state_id);
            let key = (self.partition_key)(meta);
            partition.entry(key).or_default().insert(state_id);
        }

        partition.into_iter().map(|(_, x)| x).collect()
    }

    /// 通过反向边，获取C的前驱节点
    fn find_predecessors_by_char(
        &self,
        split_set: &BTreeSet<StateID>,
        c: usize,
    ) -> BTreeSet<StateID> {
        let mut reach = BTreeSet::new();
        for &state_id in split_set {
            // 获取所有反向边
            let set = match self.reverse_table.get(&(state_id, c)) {
                None => continue,
                Some(x) => x,
            };

            reach.extend(set);
        }

        reach
    }

    // 对Hopcroft内partition worklist进行更新
    fn do_partition_update(
        predecessors: &BTreeSet<StateID>,
        partition: &mut HashSet<BTreeSet<StateID>>,
        worklist: &mut HashSet<BTreeSet<StateID>>,
    ) {
        let mut update = Vec::new();

        // 不能遍历更新，采用延迟更新
        for set in partition.iter() {
            let intersection: BTreeSet<_> = set.intersection(&predecessors).cloned().collect();
            let difference: BTreeSet<_> = set.difference(&predecessors).cloned().collect();

            if !intersection.is_empty() && !difference.is_empty() {
                update.push((set.clone(), intersection, difference));
            }
        }

        // 更新
        for (set, intersection, difference) in update {
            partition.remove(&set); // 更新Partition
            partition.insert(intersection.clone());
            partition.insert(difference.clone());

            if worklist.contains(&set) {
                // 更新worklist
                worklist.remove(&set);
                worklist.insert(intersection);
                worklist.insert(difference);
            } else if intersection.len() > difference.len() {
                worklist.insert(difference);
            } else {
                worklist.insert(intersection);
            }
        }
    }

    /// hopcroft 最小化等价类划分算法
    fn hopcroft(&self) -> HashSet<BTreeSet<StateID>> {
        let vec = self.init_partition(); // 初始划分
        let mut partition = HashSet::from_iter(vec.into_iter());
        let mut worklist = partition.clone();

        let sigma = self.dfa.get_stride(); // 输入符号全集

        while !worklist.is_empty() {
            let split_set = worklist
                .iter() // 当前分割集
                .next()
                .cloned()
                .and_then(|s| worklist.take(&s))
                .unwrap();

            // 遍历所有输入符号
            for c in 0..sigma {
                let predecessors = self.find_predecessors_by_char(&split_set, c); // 前驱节点

                // 跳过无效前驱节点
                if predecessors.is_empty() {
                    continue;
                }

                // 分割 Partition 更新 WorkList
                Self::do_partition_update(&predecessors, &mut partition, &mut worklist);
            }
        }

        partition
    }

    // 构建划分表 旧状态 -> 新状态
    fn build_partition_table(&mut self, partition: HashSet<BTreeSet<StateID>>) -> Vec<StateID> {
        let mut table = Vec::new();
        table.resize(self.states.len(), 0);

        for set in partition {
            let new_state = self.next_id();
            for old_state in set {
                table[old_state] = new_state;
            }
        }

        table
    }

    // 构建转移表
    fn build_transaction_table(
        &self,
        partition: &Vec<StateID>,
    ) -> HashMap<(StateID, ClassID), StateID> {
        let mut transaction_table = HashMap::new();

        for (from, symbols) in self.symbol_table.iter().enumerate() {
            for &symbol in symbols {
                let to = match self.dfa.find_next(from, symbol) {
                    Some(to) => to,
                    None => panic!("Bug!!"),
                };
                // 将旧边旧点翻译为新边新点
                transaction_table.insert((partition[from], symbol), partition[to]);
            }
        }

        transaction_table
    }

    /// 构建DFA
    fn build_dfa(
        &self,
        partition_table: Vec<StateID>,
        transaction_table: HashMap<(StateID, ClassID), StateID>,
    ) -> DFA {
        let reverse_partition_table = self.build_reverse_partition_table(&partition_table);

        let mut dfa = DFA::new(
            partition_table[self.dfa.get_init_state()],
            reverse_partition_table.len(),
            self.dfa.get_stride(),
        );
        for (new_state, old_states) in reverse_partition_table.into_iter().enumerate() {
            if old_states.is_empty() {
                continue;
            }
            let old_state = (self.priority_status)(&self.dfa, old_states);
            dfa.add_state(new_state, self.dfa.get_meta(old_state).clone())
        }

        for ((from, symbol), to) in transaction_table {
            dfa.add_transition((from, symbol, to));
        }

        dfa
    }

    // 反向构建partition_table 新状态 -> {旧状态}
    fn build_reverse_partition_table(
        &self,
        partition_table: &Vec<StateID>,
    ) -> Vec<BTreeSet<StateID>> {
        let states: HashSet<StateID> = partition_table.iter().copied().collect();
        let mut reverse_partition_table = Vec::new();
        reverse_partition_table.resize_with(states.len(), BTreeSet::new);

        for (new_state, &old_state) in partition_table.iter().enumerate() {
            reverse_partition_table[old_state].insert(new_state);
        }

        reverse_partition_table
    }
}
