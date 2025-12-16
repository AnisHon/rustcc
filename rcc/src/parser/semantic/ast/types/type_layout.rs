use crate::parser::ast::types::{Type, TypeKind};

impl Type {

    pub fn align(&self) -> Option<u64> {
        use crate::parser::semantic::ast::types::type_struct::TypeKind::*;
        match &self.kind {
            Void | Unknown  => None,
            Integer{ .. } => Some(4),
            Floating{ .. } => Some(4),
            Pointer{ .. } => Some(8),
            Array{ size, elem_ty } =>
                elem_ty.upgrade().unwrap().align().map(|x| x * size.get_static()),
            Function{ .. } => Some(8),
            Struct{ fields, .. } => Some(
                fields.iter()
                    .map(|x| x.ty.upgrade().unwrap().align().unwrap())
                    .sum()
            ),
            StructRef{ .. } => None,
            Union{ fields, .. } => fields.iter().map(|x| x.ty.upgrade().unwrap().align().unwrap()).max(),
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