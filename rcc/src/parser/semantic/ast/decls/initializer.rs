use crate::parser::ast::ExprKey;
use crate::parser::ast::decls::decl::InitializerList;

#[derive(Debug, Clone)]
pub enum Initializer {
    Expr(ExprKey),
    InitList { inits: InitializerList },
}
