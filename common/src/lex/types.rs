pub type StateID = usize;

pub type ClassID = usize;

///
/// 状态元信息
///
/// # Members
/// - `id`: 用于终结节点归类，只有终结节点需要
/// - `name`: 用于标记终结节点的名字
/// - `priority`: 用于终结节点之间确定优先级
/// - `terminate`: 是否是终结节点
///
#[derive(Debug, Clone)]
pub struct StateMeta {
    pub id: Option<StateID>,
    pub name: Option<String>,
    pub priority: Option<usize>,
    pub terminate: bool, // 其他
}

impl Default for StateMeta {
    fn default() -> StateMeta {
        StateMeta {
            id: None,
            priority: None, // 最大表示没有优先级
            name: None,
            terminate: false,
        }
    }
}
