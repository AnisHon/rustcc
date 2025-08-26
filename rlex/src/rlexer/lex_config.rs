// 词法分析器 结构
#[derive(Debug)]
pub struct LexStruct {
    pub name: String,
    pub regex: String,
    pub skip: bool,
}
