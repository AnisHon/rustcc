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
pub struct DFA {
    states: Vec<StateMeta>,
    transitions: Vec<Vec<Option<StateID>>>,
    start: StateID,
    stride: usize
}
impl DFA {

    ///
    /// 构造函数
    /// # 参数
    /// start: 初始状态ID
    /// state_sz: state数量
    /// stride: 最大转移数量
    /// 
    pub fn new(start: StateID, states_sz: usize, stride: usize,) -> DFA {
        let mut states = vec![]; // 全部初始化为normal
        let mut transitions = vec![];

        for _ in 0..states_sz {     // 初始化所有state
            states.push(StateMeta::default());
        }
        for _ in 0..states_sz {     // 初始化状态矩阵
            transitions.push(vec![None; stride]);
        }

        DFA { states, transitions, start, stride }
    }
    
    pub fn start(&self) -> StateID {
        self.start
    }

    pub fn get_meta(&self, state_id: StateID) -> &StateMeta {
        assert!(self.states.len() > state_id);
        &self.states[state_id]
    }

    pub fn get_meta_mut(&mut self, state_id: StateID) -> &mut StateMeta {
        assert!(self.states.len() > state_id);
        &mut self.states[state_id]
    }


    ///
    /// 转移操作 (state_id class_id) -> state_id
    ///
    pub fn find_next(&self, state_id: StateID, class_id: usize) -> Option<StateID> {
        assert!(self.states.len() > state_id);
        assert!(self.stride > state_id);
        self.transitions[state_id][class_id]
    }

    ///
    /// 添加转移边，(origin, class) -> dest
    /// 如果存在，则会覆盖
    ///
    pub fn add_transition(&mut self, (origin, class, dest): (StateID, ClassID, StateID)) -> &mut Self {
        assert!(self.states.len() > origin);
        assert!(self.states.len() > dest);
        self.transitions[origin][class] = Some(dest);
        
        // 没什么意义就是方便我测试的时候链式调用
        self
    }





}


