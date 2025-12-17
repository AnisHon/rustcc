use std::hash::{Hash, Hasher};
use crate::parser::ast::types::type_struct::TypeKey;
use crate::parser::common::Ident;
use crate::parser::semantic::comp_ctx::CompCtx;

///
/// # Members
/// - `name`: 成员名
/// - `ty`: 成员类型
/// - `bit_field`: 位域
/// - `offset`: 偏移量
///
#[derive(Debug, Clone)]
pub struct RecordField {
    pub name: Option<Ident>,
    pub ty: TypeKey,
    pub bit_field: Option<u64>,
    pub offset: u64,
}

impl RecordField {
    
    pub fn to_code(&self, ctx: &CompCtx) -> String { // todo 抽到外面
        let mut code = String::new();

        let ty = ctx.get_type(self.ty).to_code(ctx);
        let name = self.name.as_ref().map(|x| x.symbol.get()).unwrap_or_default();

        code.push_str(&ty);
        code.push(' ');
        code.push_str(name);

        match self.bit_field.map(|x| x.to_string()) {
            None => {},
            Some(x) => {
                code.push_str(": ");
                code.push_str(&x);
            }
        }

        code.push(';');
        code
    }
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

///
/// # Members
/// - `name`: 枚举名
/// - `value`: 枚举值
///
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumField {
    pub name: Ident,
    pub value: u64,
}

impl EnumField {
    pub fn to_code(&self) -> String {
        let mut code = String::new();
        code.push_str(self.name.symbol.get());
        code.push('=');
        code.push_str(&self.value.to_string());
        code
    }
}