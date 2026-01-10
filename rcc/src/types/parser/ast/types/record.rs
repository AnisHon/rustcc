use crate::types::lex::token_kind::Symbol;
use crate::types::parser::ast::TypeKey;
use crate::parser::comp_ctx::CompCtx;
use std::hash::{Hash, Hasher};

///
/// # Members
/// - `name`: 成员名
/// - `ty`: 成员类型
/// - `bit_field`: 位域
/// - `offset`: 偏移量
///
#[derive(Debug, Clone)]
pub struct RecordField {
    pub name: Option<Symbol>,
    pub ty: TypeKey,
    pub bit_field: Option<u128>,
    pub offset: u64,
}

impl PartialEq for RecordField {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.ty == other.ty
    }
}

impl Eq for RecordField {}

impl Hash for RecordField {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        &self.ty.hash(state);
    }
}
