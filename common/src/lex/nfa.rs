use crate::lex::types::{ClassID, StateID, StateMeta};
use std::collections::{BTreeSet, HashMap};

/// 由于NFA有空转移，所以需要单独设计Epsilon
#[derive(Debug, Ord, Eq, Hash, PartialEq, Clone, Copy, PartialOrd)]
pub enum NFASymbol {
    Epsilon,
    ClassID(ClassID),
}

/// NFA不是最终目标，支持合并，因此NFA使用较为灵活的HashMap而不是Vec，维护一个id变量防止冲突
/// 低于StatusEdge，TerminateState 使用 BTreeSet 可以更方便一点
#[derive(Debug)]
pub struct NFA {
    status: HashMap<StateID, StateMeta>, // nfa 的StateID相对离散使用HashMap
    edges: HashMap<(StateID, NFASymbol), BTreeSet<StateID>>, // (status + 转移边) -> 多个status
    status_edges: HashMap<StateID, BTreeSet<NFASymbol>>, // status -> 所有对应转移边，方便转换NFA扫描
    initial_state: StateID,                              // 初始节点
    terminated_states: BTreeSet<StateID>,                // 终结节点
}

impl NFA {
    /// 默认terminate_node为initial_state
    pub fn new(initial_state: StateID) -> Self {
        NFA {
            status: HashMap::new(),
            edges: HashMap::new(),
            status_edges: HashMap::new(),
            initial_state,
            terminated_states: BTreeSet::new(),
        }
    }

    /// 检查是否存在state
    pub fn exist(&self, state_id: StateID) -> bool {
        self.status.contains_key(&state_id)
    }

    pub fn all_exists(&self, state_ids: &BTreeSet<StateID>) -> bool {
        for &x in state_ids {
            if !self.exist(x) {
                return false;
            }
        }
        true
    }

    /// 获取初始状态
    pub fn get_init_state(&self) -> StateID {
        self.initial_state
    }

    /// 获取 status 的meta信息
    pub fn get_status(&self, state: StateID) -> &StateMeta {
        &self.status[&state]
    }

    /// 获取 status 的meta信息
    pub fn get_status_mut(&mut self, state: StateID) -> &mut StateMeta {
        self.status.get_mut(&state).unwrap()
    }

    pub fn get_status_ids(&self) -> impl Iterator<Item = &StateID> {
        self.status.keys()
    }

    pub fn get_symbols(&self, state: StateID) -> &BTreeSet<NFASymbol> {
        self.status_edges.get(&state).unwrap()
    }

    pub fn get_terminated_states(&self) -> &BTreeSet<StateID> {
        &self.terminated_states
    }

    pub fn find_next(&self, state_id: StateID, symbol: NFASymbol) -> Option<&BTreeSet<StateID>> {
        self.edges.get(&(state_id, symbol))
    }

    /// 克隆，并且根据init_state自动计算偏移量
    pub fn clone(&self, initial_state: usize) -> NFA {
        assert!(initial_state >= self.initial_state);
        let mut nfa = NFA::new(initial_state);
        let offset = initial_state - self.initial_state;

        self.status.iter().for_each(|(id, meta)| {
            nfa.status.insert(*id + offset, meta.clone());
        });

        self.edges.iter().for_each(|((from, symbol), to)| {
            let to = to.iter().map(|x| *x + offset).collect();
            nfa.edges.insert((*from + offset, *symbol), to);
        });

        self.terminated_states.iter().for_each(|id| {
            nfa.terminated_states.insert(*id + offset);
        });

        nfa
    }

    pub fn set_terminate(&mut self, state: StateID) {
        assert!(self.status.contains_key(&state)); // 必须存在
        self.get_status_mut(state).terminate = true;
        self.terminated_states.insert(state);
    }

    /// 添加一个状态
    pub fn add_state(&mut self, state: StateID, meta: StateMeta) -> &mut Self {
        assert!(!self.status.contains_key(&state)); // 不允许覆盖
        self.status.insert(state, meta);
        self.status_edges.insert(state, BTreeSet::new()); // 为其初始化数组

        self
    }

    /// 添加一条边
    pub fn add_edge(&mut self, from: StateID, edge: NFASymbol, to: StateID) -> &mut Self {
        assert!(self.status.contains_key(&from)); // 不存在一定有错 
        assert!(self.status.contains_key(&to));

        self.edges // 添加边
            .entry((from, edge))
            .or_default()
            .insert(to);

        self.status_edges // 维护状态边映射
            .get_mut(&from)
            .unwrap()
            .insert(edge);

        self
    }

    /// 合并NFA，返回被合并方的初始状态号
    pub fn merge(&mut self, other: NFA) -> StateID {
        // todo 由于other是移动后的，因此可以此处应该判断双方谁合并谁高效

        // 合并 status
        for (sid, meta) in other.status.into_iter() {
            if self.status.contains_key(&sid) {
                panic!("StateID conflict when merging: {}", sid);
            }
            self.status.insert(sid, meta);
        }

        // 合并 edges
        for ((from, class), targets) in other.edges.into_iter() {
            let entry = self
                .edges
                .entry((from, class))
                .or_default();

            for to in targets {
                if !self.status.contains_key(&to) {
                    panic!(
                        "Target state {} not found in status map during edge merge",
                        to
                    );
                }

                entry.insert(to);
            }
        }

        // 合并 状态-边映射
        for (sid, class_ids) in other.status_edges.into_iter() {
            self.status_edges
                .entry(sid)
                .or_default()
                .extend(class_ids.iter().copied());
        }

        // 合并 终态节点
        self.terminated_states.extend(other.terminated_states);

        // 返回对方初始状态
        other.initial_state
    }

    /// 根据init_state合并自动计算偏移
    pub fn merge_offset(&mut self, other: &NFA, init_state: StateID) -> StateID {
        let min_state = other.status.keys().min().expect("Empty NFA");

        let offset = init_state - min_state;

        // 合并 status
        for (sid, meta) in other.status.iter() {
            let sid = *sid + offset;
            if self.status.contains_key(&sid) {
                panic!("StateID conflict when merging: {}", sid);
            }
            self.status.insert(sid, meta.clone());
        }

        // 合并 edges
        for ((from, class), targets) in other.edges.iter() {
            let from = *from + offset;
            let targets = targets.iter().map(|x| *x + offset);

            let entry = self
                .edges
                .entry((from, *class))
                .or_default();

            for to in targets {
                if !self.status.contains_key(&to) {
                    panic!(
                        "Target state {} not found in status map during edge merge",
                        to
                    );
                }

                entry.insert(to);
            }
        }

        // 合并 状态-边映射
        for (sid, class_ids) in other.status_edges.iter() {
            let sid = *sid + offset;
            self.status_edges
                .entry(sid)
                .or_default()
                .extend(class_ids.iter().copied());
        }

        self.terminated_states
            .extend(other.terminated_states.iter().map(|x| *x + offset));

        other.initial_state + offset
    }
}
