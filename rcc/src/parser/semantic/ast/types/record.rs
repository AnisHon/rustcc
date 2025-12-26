use crate::lex::types::token_kind::Symbol;
use crate::parser::ast::TypeKey;
use crate::parser::semantic::comp_ctx::CompCtx;
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

impl RecordField {
    pub fn to_code(&self, ctx: &CompCtx) -> String {
        // todo 抽到外面
        let mut code = String::new();

        let ty = ctx.type_ctx.get_type(self.ty).to_code(ctx);
        let name = self.name.as_ref().map(|x| x.get()).unwrap_or_default();

        code.push_str(&ty);
        code.push(' ');
        code.push_str(name);

        match self.bit_field.map(|x| x.to_string()) {
            None => {}
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
