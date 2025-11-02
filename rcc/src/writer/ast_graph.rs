use petgraph::Graph;
use petgraph::graph::DiGraph;
use crate::parser::ast::decl::{Decl, DeclKind, Initializer};
use crate::parser::ast::expr::Expr;
use crate::parser::ast::stmt::Stmt;
use crate::parser::ast::visitor::Visitor;

pub struct AstGraph {
    tree: DiGraph<String, ()>,
}

impl AstGraph {
    pub fn new() -> Self {
        let tree = DiGraph::new();
        Self {
            tree
        }
    }
}

impl AstGraph {
    fn visit_initializer(&mut self, initializer: &Initializer) {
        match initializer {
            Initializer::Expr(x) => self.visit_expr(&x),
            Initializer::InitList { inits, .. } => {
                let inits: Vec<_> = inits.inits.iter().map(|x| self.visit_initializer(x)).collect();
            }
        }
    }
}

impl Visitor for AstGraph {
    fn visit_decl(&mut self, decl: &Decl) {
        let name = decl.name.as_ref().map(|x| x.symbol.get()).unwrap_or_default();
        match &decl.kind {
            DeclKind::ParamVar => {
                let code = decl.ty.to_code();

            }
            DeclKind::VarInit { init, .. } => {
                let init = init.as_ref().map(|x| self.visit_initializer(x));
            }
            DeclKind::FuncRef { ret_ty, params, is_variadic } => {
                let ret = ret_ty.to_code();
                for x in params {
                    self.visit_decl(x);
                }
            }
            DeclKind::Func { ret_ty, params, is_variadic, body } => {
                let ret = ret_ty.to_code();
                for x in params {
                    self.visit_decl(x);
                }
                self.visit_stmt(body);
            }
            DeclKind::RecordField { bit_field, .. } => {
                bit_field.as_ref().map(|x| self.visit_expr(x));

            }
            DeclKind::Record { kind, fields, .. } => {

            }
            DeclKind::RecordRef { kind } => {}
            DeclKind::EnumField { expr, .. } => {}
            DeclKind::Enum { enums, .. } => {}
            DeclKind::EnumRef { .. } => {

            }
        }
    }

    fn visit_expr(&mut self, expr: &Expr) {

    }

    fn visit_stmt(&mut self, stmt: &Stmt) {

    }
}