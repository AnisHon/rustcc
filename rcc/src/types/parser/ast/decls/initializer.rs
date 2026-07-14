use crate::parser::comp_ctx::CompCtx;
use crate::types::parser::ast::ExprKey;
use crate::types::span::Span;

#[derive(Debug, Clone)]
pub enum Initializer {
    Expr(ExprKey),
    InitList { inits: InitializerList },
}

impl Initializer {
    pub fn get_span(&self, ctx: &CompCtx) -> Span {
        match self {
            Initializer::Expr(x) => ctx.get_expr(*x).span,
            Initializer::InitList { inits } => inits.span,
        }
    }
}

#[derive(Clone, Debug)]
pub struct InitializerList {
    pub inits: Vec<Initializer>,
    pub span: Span,
}

impl InitializerList {
    pub fn new() -> Self {
        Self {
            inits: Vec::new(),
            span: Span::default(),
        }
    }
}
