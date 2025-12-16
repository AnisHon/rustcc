use slotmap::SlotMap;
use crate::parser::ast::decl::{Decl, DeclKey};
use crate::parser::ast::exprs::{Expr, ExprKey};
use crate::parser::ast::types::{Type, TypeKey};

macro_rules! make_get {
    ($get:ident, $get_mut:ident, $field:ident, $key_ty:ty, $ty:ty) => {
        fn $get(&self, key: $key_ty) -> &$ty {
            self.$field.get(key).expect(concat!("wrong key in", stringify!($field)))
        }

        fn $get_mut(&mut self, key: $key_ty) -> &mut $ty {
            self.$field.get_mut(key).expect(concat!("wrong key in", stringify!($field)))
        }
    };
}
struct CompCtx {
    pub decls: SlotMap<DeclKey, Decl>,
    pub exprs: SlotMap<ExprKey, Expr>,
    pub types: SlotMap<TypeKey, Type>,
}

impl CompCtx {
    make_get!(get_decl, get_decl_mut, decls, DeclKey, Decl);
    make_get!(get_expr, get_expr_mut, exprs, ExprKey, Expr);
    make_get!(get_type, get_type_mut, types, TypeKey, Type);
}