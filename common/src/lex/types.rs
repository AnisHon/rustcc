pub type StateID = usize;

pub type ClassID = usize;

///
/// 状态元信息
///
#[derive(Debug, Clone)]
pub struct StateMeta {
    pub terminate: bool, // 其他
}

impl StateMeta {
    pub fn default() -> StateMeta {
        StateMeta { terminate: false }
    }
}
