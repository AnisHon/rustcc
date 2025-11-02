use crate::parser::types::ast::expr::{Expr, ExprKind};
use crate::parser::types::sema::Sema;
use crate::types::span::Span;

impl Sema {

    /// 构建expression
    pub fn make_expr(&mut self, kind: ExprKind, span: Span) -> Box<Expr> {
        // todo 需要实现，目前默认unknown
        let ty = self.type_context.get_unknown_type();
        Box::new(Expr {
            kind,
            ty,
            span
        })
    }

}