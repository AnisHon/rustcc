use crate::types::parser::ast::ExprKey;
use crate::types::span::Span;

#[derive(Debug, Clone)]
pub enum Initializer {
    Expr(ExprKey),
    InitList { inits: InitializerList },
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
