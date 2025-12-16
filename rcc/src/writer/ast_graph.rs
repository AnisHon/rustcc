use std::rc::Rc;
use crate::parser::ast::decl::{DeclGroup, DeclKind, DeclRef, Initializer};
use crate::parser::ast::expr::{Expr, ExprKind};
use crate::parser::ast::stmt::{Stmt, StmtKind};
use crate::parser::ast::visitor::Visitor;
use petgraph::graph::{DiGraph, NodeIndex};
use crate::parser::ast::func::{ExternalDecl, FuncDef, TranslationUnit};

pub struct AstGraph {
    pub tree: DiGraph<String, ()>,
    current: NodeIndex,
}

impl AstGraph {
    pub fn new() -> Self {
        let tree = DiGraph::new();
        Self {
            tree,
            current: NodeIndex::default()
        }
    }

    /// 返回旧的节点
    fn make_node(&mut self, name: String) -> NodeIndex {
        let prev = self.current;
        self.current = self.connect_node(name);
        prev
    }

    fn connect_node(&mut self, name: String) -> NodeIndex {
        let node_idx = self.tree.add_node(name);
        self.tree.add_edge(self.current, node_idx, ());
        node_idx
    }
}

impl AstGraph {
    fn visit_initializer(&mut self, initializer: &mut Initializer) {
        let prev = self.make_node("initializer".to_owned());
        match initializer {
            Initializer::Expr(x) => self.visit_expr(x),
            Initializer::InitList { inits, .. } => {
                inits.inits.iter_mut().for_each(|x| self.visit_initializer(x))
            }
        }
        self.current = prev;
    }
}

impl Visitor for AstGraph {
    fn walk_translation_unit(&mut self, unit: &mut TranslationUnit) {
        let prev = self.make_node("translation_unit".to_string());
        for ext_decl in unit {
            self.walk_external_decl(ext_decl);
        }
        self.current = prev;
    }

    fn walk_external_decl(&mut self, decl: &mut ExternalDecl) {
        let prev = self.make_node("external_decl".to_string());
        match decl {
            ExternalDecl::FunctionDefinition(x) => self.walk_func_def(x),
            ExternalDecl::Declaration(x) => self.walk_decl_group(x)
        }
        self.current = prev;
    }

    fn walk_func_def(&mut self, decl: &mut FuncDef) {
        let prev = self.make_node("func_def".to_string());
        self.visit_decl(Rc::clone(&decl.decl));
        self.visit_stmt(&mut decl.body);
        self.current = prev;
    }

    fn walk_decl_group(&mut self, decl_group: &DeclGroup) {
        let prev = self.make_node("decl_group".to_string());
        for x in decl_group.decls.iter().cloned() {
            self.visit_decl(x);
        }
        self.current = prev;
    }

    fn visit_decl(&mut self, decl: DeclRef) {
        let prev;
        let mut decl = decl.borrow_mut();
        let name = decl.name.as_ref().map(|x| x.symbol.get()).unwrap_or_default();
        self.connect_node(name.to_string());

        match &mut decl.kind {
            DeclKind::ParamVar => {
                let ty = decl.ty.to_code();
                prev = self.make_node(ty);
            }
            DeclKind::VarInit { init, .. } => {
                prev = self.make_node("var_init".to_owned());
                if let Some(x) = init {
                    self.visit_initializer(x);
                }
            }
            DeclKind::FuncRef {  } => {
                prev = self.make_node("func_ref".to_owned());
                // let ret = ret_ty.to_code();
                // self.connect_node(ret);
                // for x in params.iter() {
                //     self.visit_decl(Rc::clone(x));
                // }
            }
            DeclKind::Func {body } => {
                prev = self.make_node("func_def".to_owned());
                // let ret = ret_ty.to_code();
                // self.connect_node(ret);
                // for x in params.iter() {
                //     self.visit_decl(Rc::clone(x));
                // }
                self.visit_stmt(body);
            }
            DeclKind::RecordField { bit_field, .. } => {
                prev = self.make_node("record_field".to_owned());
                if let Some(x) =bit_field.as_mut() {
                   self.visit_expr(x)
                };
            }
            DeclKind::Record { fields, .. } => {
                prev = self.make_node("record_def".to_owned());
                fields.iter_mut().for_each(|x| self.walk_decl_group(x))
            }
            DeclKind::RecordRef { kind } => {
                prev = self.make_node("record_ref".to_owned());
            }
            DeclKind::EnumField { expr, .. } => {
                prev = self.make_node("enum_field".to_owned());
                if let Some(x) = expr {
                    self.visit_expr(x)
                }
            }
            DeclKind::Enum { enums, .. } => {
                prev = self.make_node("enum_def".to_owned());
                enums.iter().for_each(|x| self.visit_decl(Rc::clone(x)))
            }
            DeclKind::EnumRef { .. } => {
                prev = self.make_node("enum_ref".to_owned());
            }
            DeclKind::TypeDef { } => {
                prev = self.make_node("typedef".to_owned());
            }
        }
        self.current = prev;
    }

    fn visit_expr(&mut self, expr: &mut Expr) {
        let prev;
        match &mut expr.kind {
            ExprKind::DeclRef(x) => {
                prev = self.make_node(format!("var {}", x.symbol.get()));
            }
            ExprKind::Constant(x) => {
                prev = self.make_node(format!("{:?}", x));
            }
            ExprKind::Paren { expr, .. } => {
                prev = self.make_node("exprs".to_owned());
                self.visit_expr(expr)
            }
            ExprKind::ArraySubscript { base, index, .. } => {
                prev = self.make_node("arr_acc".to_owned());
                self.visit_expr(base);
                self.visit_expr(index);
            }
            ExprKind::Call { base, params, .. } => {
                prev = self.make_node("arr_acc".to_owned());
                self.visit_expr(base);
                params.exprs.iter_mut().for_each(|x| self.visit_expr(x));
            }
            ExprKind::MemberAccess { base, field, .. } => {
                prev = self.make_node("mem_acc".to_owned());
                self.visit_expr(base);
                self.connect_node(field.get().to_owned());
            }
            ExprKind::SizeofExpr { expr, .. } => {
                prev = self.make_node("sizeof".to_owned());
                self.visit_expr(expr);
            }
            ExprKind::SizeofType { ty, .. } => {
                prev = self.make_node("sizeof".to_owned());
                self.connect_node(ty.to_code());
            }
            ExprKind::Unary { op, rhs, .. } => {
                prev = self.make_node(format!("{:?}", op.kind));
                self.visit_expr(rhs)
            }
            ExprKind::Assign { op, lhs, rhs, .. }  => {
                prev = self.make_node(format!("{:?}", op.kind));
                self.visit_expr(lhs);
                self.visit_expr(rhs);
            }
            ExprKind::Binary { op, lhs, rhs, .. } => {
                prev = self.make_node(format!("{:?}", op.kind));
                self.visit_expr(lhs);
                self.visit_expr(rhs);
            }
            ExprKind::Cast { ty,  expr, .. } => {
                prev = self.make_node("cast".to_owned());
                self.connect_node(ty.to_code());
                self.visit_expr(expr)
            }
            ExprKind::Ternary { cond, then_expr, else_expr, .. } => {
                prev = self.make_node("cond_expr".to_owned());
                self.visit_expr(cond);
                self.visit_expr(then_expr);
                self.visit_expr(else_expr);
            }
        }
        self.current = prev;
    }

    fn visit_stmt(&mut self, stmt: &mut Stmt) {
        let prev;
        match &mut stmt.kind {
            StmtKind::Expr { expr, .. } => {
                prev = self.make_node("expr_stmt".to_owned());
                if let Some(x) = expr {
                    self.visit_expr(x)
                }
            }
            // StmtKind::Decl { .. } => {}
            // StmtKind::Label { .. } => {}
            // StmtKind::Case { .. } => {}
            // StmtKind::Default { .. } => {}
            // StmtKind::IfElse { .. } => {}
            // StmtKind::Switch { .. } => {}
            // StmtKind::While { .. } => {}
            // StmtKind::DoWhile { .. } => {}
            // StmtKind::For { .. } => {}
            // StmtKind::Goto { .. } => {}
            // StmtKind::Continue { .. } => {}
            // StmtKind::Break { .. } => {}
            StmtKind::Return { expr, .. } => {
                prev = self.make_node("compound_stmt".to_owned());
                if let Some(x) = expr {
                    self.visit_expr(x);
                }
            }
            StmtKind::Compound { stmts, .. } => {
                prev = self.make_node("compound_stmt".to_owned());
                stmts.iter_mut().for_each(|x| self.visit_stmt(x));
            }
            _ => {
                prev = self.make_node("UnImplement".to_owned());
            }
        }
        self.current = prev;
    }
}