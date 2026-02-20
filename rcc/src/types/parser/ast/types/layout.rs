use crate::parser::comp_ctx::CompCtx;
use crate::types::parser::ast::types::Type;

#[derive(Debug, Clone)]
pub struct TypeLayout {
    pub size: usize,
    pub align: usize,
}

impl TypeLayout {
    pub fn new(ctx: &CompCtx, ty: &Type) -> Self {
        Self { size, align }
    }
}
