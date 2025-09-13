pub type StateID = usize;

pub type ClassID = usize;

///
/// 状态元信息
///
#[derive(Debug, Clone)]
pub struct StateMeta {
    pub id: StateID,
    pub name: Option<String>,
    pub priority: usize,
    pub terminate: bool, // 其他
}

impl Default for StateMeta {
    fn default() -> StateMeta {
        StateMeta {
            id: StateID::MAX,
            priority: StateID::MAX,
            name: None,
            terminate: false,
        }
    }
}
