use crate::parser::{ast::types::{Type, TypeKind}, semantic::comp_ctx::CompCtx};

impl Type {

    // todo: type is immutable, that should store in typestruct 
    pub fn align(&self, ctx: &CompCtx) -> Option<u64> {
        use crate::parser::semantic::ast::types::type_struct::TypeKind::*;
        match &self.kind {
            Void | Unknown  => None,
            Integer{ .. } => Some(4),
            Floating{ .. } => Some(4),
            Pointer{ .. } => Some(8),
            Array{ size, elem_ty } =>
                ctx.get_type(*elem_ty).align(ctx).map(|x| x * size.get_static()),
            Function{ .. } => Some(8),
            Struct{ fields, .. } => { // less element size
                let mut sum= 1u64; // at least 1 byte for empty struct
                for item in fields {
                    match ctx.get_type(item.ty).align(ctx) {
                        Some(x) => sum += x,
                        None => return None, // wrong size, just return none
                    }
                }
                Some(sum)
            },
            StructRef{ .. } => None,
            Union{ fields, .. } => { // max element size 
                let mut sum= 1u64; // at least 1 byte for empty union
                for item in fields {
                    match ctx.get_type(item.ty).align(ctx) {
                        Some(x) => sum = max(sum, x), 
                        None => return None, // wrong size, just return none
                    }
                }
                Some(sum)
            }
            UnionRef{ .. } => None,
            Enum{ .. } => Some(8),
            EnumRef{ .. } => None,
        }
    }

    pub fn sizeof(&self) -> u64 {
        use TypeKind::*;
        match &self.kind {
            Void => 1,
            Integer { size, .. } => size.sizeof(),
            Floating { size, .. } => size.sizeof(),
            Pointer { .. } => 8,
            Array { size, elem_ty } => todo!(),
            Function { .. } => 1,
            Struct { size, .. } => *size,
            StructRef { .. } => 1,
            Union { size,.. } => *size,
            UnionRef { .. } => 1,
            Enum { .. } => 4,
            EnumRef { .. } => 1,
            Unknown => 0,
        }
    }
}