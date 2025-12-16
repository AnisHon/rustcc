use crate::parser::ast::types::{Type, TypeKind};

impl Type {
    pub fn is_unknown(&self) -> bool {
        matches!(&self.kind, TypeKind::Unknown)
    }

    pub fn is_arithmetic(&self) -> bool {
        matches!(self.kind, TypeKind::Integer{ .. } | TypeKind::Floating{ .. })
    }

    pub fn is_pointer(&self) -> bool {
        matches!(self.kind, TypeKind::Pointer{ .. } | TypeKind::Array{ .. })
    }

    pub fn is_scalar(&self) -> bool {
        self.is_pointer() || self.is_arithmetic()
    }

    pub fn is_void_ptr(&self) -> bool {
        match &self.kind {
            TypeKind::Pointer { elem_ty } => {
                match elem_ty.upgrade().unwrap().kind {
                    TypeKind::Void => true,
                    _ => false,
                }
            }
            _ => false,
        }
    }
}