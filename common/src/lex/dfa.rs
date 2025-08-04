use crate::lex::types::{ClassID, StateID, StateMeta};

/// DFA数据定义，所有节点使用StateID类型定义
/// 平衡性能和空间，使用二维矩阵，使用等价类后class_id大小可控
///
/// # 成员
/// states: 所有状态信息表
/// transitions: 所有转移边 state = [state][class_id],
/// start: 起始状态
/// stride: 矩阵行大小
///
#[derive(Debug)]
pub struct DFA {
    states: Vec<Option<StateMeta>>,
    transitions: Vec<Vec<Option<StateID>>>,
    init_state: StateID,
    stride: usize,
}
impl DFA {
    ///
    /// 构造函数
    /// # 参数
    /// start: 初始状态ID
    /// state_sz: state数量
    /// stride: 最大转移数量
    ///
    pub fn new(init_state: StateID, states_sz: usize, stride: usize) -> DFA {
        let mut states = vec![]; // 全部初始化为normal
        let mut transitions = vec![];

        for _ in 0..states_sz {
            // 初始化所有state
            states.push(None);
        }
        for _ in 0..states_sz {
            // 初始化状态矩阵
            transitions.push(vec![None; stride]);
        }

        DFA {
            states,
            transitions,
            init_state,
            stride,
        }
    }

    pub fn get_init_state(&self) -> StateID {
        self.init_state
    }

    pub fn add_state(&mut self, state_id: StateID, meta: StateMeta) {
        assert!(state_id < self.states.len());
        self.states[state_id] = Some(meta);
    }

    /// 状态数量
    pub fn size(&self) -> usize {
        self.states.len()
    }

    pub fn get_stride(&self) -> usize {
        self.stride
    }

    pub fn is_exist(&self, state_id: StateID) -> bool {
        self.states[state_id].is_some()
    }

    pub fn get_meta(&self, state_id: StateID) -> &StateMeta {
        assert!(self.states.len() > state_id);
        self.states[state_id].as_ref().unwrap()
    }

    pub fn get_meta_mut(&mut self, state_id: StateID) -> &mut StateMeta {
        assert!(self.states.len() > state_id);
        self.states[state_id].as_mut().unwrap()
    }

    pub fn get_symbols(&self, state_id: StateID) -> Vec<ClassID> {
        self.transitions[state_id]
            .iter()
            .enumerate()
            .filter(|(_, meta)| meta.is_some())
            .map(|(i, _)| i)
            .collect()
    }

    ///
    /// 转移操作 (state_id class_id) -> state_id
    ///
    pub fn find_next(&self, state_id: StateID, class_id: usize) -> Option<StateID> {
        assert!(self.states.len() > state_id);
        assert!(self.stride > class_id);

        self.transitions[state_id][class_id]
    }

    ///
    /// 添加转移边，(origin, class) -> dest
    /// 如果存在，则会覆盖
    ///
    pub fn add_transition(
        &mut self,
        (origin, class, dest): (StateID, ClassID, StateID),
    ) -> &mut Self {
        assert!(self.states.len() > origin); // 节点必须存在
        assert!(self.states.len() > dest);
        assert!(self.transitions[origin][class].is_none()); // 不允许覆盖
        self.transitions[origin][class] = Some(dest);

        // 没什么意义就是方便我测试的时候链式调用
        self
    }
}
