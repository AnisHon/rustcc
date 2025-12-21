use crate::{constant::typ::{DEFAULT_ALIGN, DEFAULT_SIZE}, parser::{ast::types::Type, comp_ctx::CompCtx}};

#[derive(Debug, Clone)]
pub struct TypeLayout {
    pub size: u64,
    pub align: u64,
}

impl TypeLayout {
    pub fn new(ctx: &CompCtx, ty: &Type) -> Self {
        let size = Self::sizeof(ty);
        let align = Self::alignof(ctx, ty).unwrap_or(DEFAULT_ALIGN);

        Self { size, align }
    }


    // todo: type is immutable, that should store in typestruct 
    pub fn alignof(ctx: &CompCtx, ty: &Type) -> Option<u64> {
        use crate::parser::semantic::ast::types::type_struct::TypeKind::*;
        match &ty.kind {
            Void | Unknown  => None,
            Integer{ .. } => Some(4),
            Floating{ .. } => Some(4),
            Pointer{ .. } => Some(8),
            Array{ size, elem_ty } =>
                todo!(),
            Function{ .. } => Some(8),
            Record {..} => todo!(),
            Enum{ .. } => Some(8),
        }
    }

    pub fn sizeof(ty: &Type) -> u64 {
        use super::TypeKind::*;
        match &ty.kind {
            Void => 1,
            Integer { size, .. } => size.sizeof(),
            Floating { size, .. } => size.sizeof(),
            Pointer { .. } => 8,
            Array { size, elem_ty } => todo!(),
            Function { .. } => 1,
            Record { .. } => todo!(), 
            Enum { .. } => 4,
            Unknown => return DEFAULT_SIZE as u64,
        }
    }
}
