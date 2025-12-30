use crate::lex::types::token_kind::Symbol;
use crate::parser::ast::DeclKey;
use rustc_hash::FxHashMap;

/// Scope 当前类型
#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub enum ScopeKind {
    Global,
    Function,
    Block,
    ParamList,
    Record,
    // Enum,
}

// 表示一个作用域
/// - `sym_ht`: symbol hash table
#[derive(Debug, Default)]
pub struct Scope {
    pub sym_ht: FxHashMap<Symbol, DeclKey>,
}

// /// 管理结构体、联合体、枚举的标签
// /// - `sym_ht`: symbol hash table
// #[derive(Default, Debug)]
// pub struct TagsScope {
//     pub sym_ht: FxHashMap<Symbol, DeclKey>,
// }

// impl TagsScope {
//     pub fn new() -> Self {
//         Self { sym_ht: FxHashMap::default() }
//     }
// }

// /// 管理结构体/联合体成员
// #[derive(Default, Debug)]
// pub struct MembersScope {
//     pub sym_ht: FxHashMap<Symbol, DeclKey>,
// }

// impl MembersScope {
//     pub fn new() -> Self {
//         Self { sym_ht: FxHashMap::default() }
//     }
// }
// /// 管理`goto`语句使用的标签
// #[derive(Default, Debug)]
// pub struct LabelsScope {
//     pub sym_ht: FxHashMap<Symbol, DeclKey>,
// }

// impl LabelsScope {
//     pub fn new() -> Self {
//         Self { sym_ht: FxHashMap::default() }
//     }
// }

// /// 管理普通标识符
// #[derive(Default, Debug)]
// pub struct IdentsScope {
//     pub sym_ht: FxHashMap<Symbol, DeclKey>,
// }

// impl IdentsScope {
//     pub fn new() -> Self {
//         Self { sym_ht: FxHashMap::default() }
//     }
// }
