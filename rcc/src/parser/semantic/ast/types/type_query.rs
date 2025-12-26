use crate::parser::ast::types::{Type, TypeKind};
use crate::parser::semantic::comp_ctx::CompCtx;

impl Type {
    pub fn is_unknown(&self) -> bool {
        matches!(&self.kind, TypeKind::Unknown)
    }

    pub fn is_arithmetic(&self) -> bool {
        matches!(
            self.kind,
            TypeKind::Integer { .. } | TypeKind::Floating { .. }
        )
    }

    pub fn is_pointer(&self) -> bool {
        matches!(self.kind, TypeKind::Pointer { .. } | TypeKind::Array { .. })
    }

    pub fn is_scalar(&self) -> bool {
        self.is_pointer() || self.is_arithmetic()
    }

    pub fn is_void_ptr(&self, ctx: &CompCtx) -> bool {
        match &self.kind {
            TypeKind::Pointer { elem_ty } => {
                matches!(ctx.type_ctx.get_type(*elem_ty).kind, TypeKind::Void)
            }
            _ => false,
        }
    }

    pub fn is_integer(&self) -> bool {
        { self.kind.is_integer() }
    }
}
