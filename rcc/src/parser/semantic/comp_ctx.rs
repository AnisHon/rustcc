use crate::err::parser_error::ParserError;
use crate::lex::token_stream::TokenStream;
use crate::parser::ast::decl::{Decl, DeclKey};
use crate::parser::ast::exprs::{Expr, ExprKey};
use crate::parser::ast::stmt::{Stmt, StmtKey};
use crate::parser::ast::types::{Type, TypeKey};
use slotmap::SlotMap;

macro_rules! make_get {
    ($get:ident, $get_mut:ident, $insert:ident, $field:ident, $key_ty:ty, $ty:ty) => {
        pub fn $get(&self, key: $key_ty) -> &$ty {
            self.$field
                .get(key)
                .expect(concat!("wrong key in", stringify!($field)))
        }

        pub fn $get_mut(&mut self, key: $key_ty) -> &mut $ty {
            self.$field
                .get_mut(key)
                .expect(concat!("wrong key in", stringify!($field)))
        }

        pub fn $insert(&mut self, value: $ty) -> $key_ty {
            self.$field.insert(value)
        }
    };
}
pub struct CompCtx {
    pub decls: SlotMap<DeclKey, Decl>,
    pub exprs: SlotMap<ExprKey, Expr>,
    pub types: SlotMap<TypeKey, Type>,
    pub stmts: SlotMap<StmtKey, Stmt>,
    pub errors: Vec<ParserError>,
    pub stream: TokenStream,
}

impl CompCtx {
    pub fn new(stream: TokenStream) -> Self {
        return Self {
            decls: SlotMap::with_key(),
            exprs: SlotMap::with_key(),
            types: SlotMap::with_key(),
            stmts: SlotMap::with_key(),
            errors: Vec::new(),
            stream,
        };
    }

    make_get!(get_decl, get_decl_mut, insert_decl, decls, DeclKey, Decl);
    make_get!(get_expr, get_expr_mut, insert_expr, exprs, ExprKey, Expr);
    make_get!(get_type, get_type_mut, insert_type, types, TypeKey, Type);
    make_get!(get_stmt, get_stmt_mut, insert_stmt, stmts, StmtKey, Stmt);
}
