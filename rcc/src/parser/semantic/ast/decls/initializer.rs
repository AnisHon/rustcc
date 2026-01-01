use crate::parser::ast::decls::decl::InitializerList;
use crate::parser::ast::ExprKey;

#[derive(Debug, Clone)]
pub enum Initializer {
    Expr(ExprKey),
    InitList { inits: InitializerList },
}

pub array
