use crate::types::parser::ast::ExprKey;
use crate::types::parser::ast::decls::decl::InitializerList;

#[derive(Debug, Clone)]
pub enum Initializer {
    Expr(ExprKey),
    InitList { inits: InitializerList },
}
