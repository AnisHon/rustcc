use crate::parser::types::ast::decl::{Decl, DeclGroup, DeclKind};
use crate::parser::types::ast::expr::Expr;
use crate::parser::types::ast::func::{ExternalDecl, FuncDef, TranslationUnit};
use crate::parser::types::ast::stmt::{Stmt, StmtKind};

pub trait Visitor {
    fn visit_translation_unit(&mut self, unit: &TranslationUnit) {
        for ext_decl in unit {
            self.visit_external_decl(ext_decl);
        }
    }

    fn visit_external_decl(&mut self, decl: &ExternalDecl) {
        match decl {
            ExternalDecl::FunctionDefinition(x) => self.visit_func_def(x),
            ExternalDecl::Declaration(x) => self.visit_decl_group(x)
        }
    }

    fn visit_func_def(&mut self, decl: &FuncDef) {
        self.visit_decl(&decl.decl);
        self.visit_stmt(&decl.body);
    }

    fn visit_decl_group(&mut self, decl_group: &DeclGroup) {
        for x in decl_group.decls.iter() {
            self.visit_decl(x);
        }
    }

    fn visit_decl(&mut self, decl: &Decl);

    fn visit_expr(&mut self, expr: &Expr);

    fn visit_stmt(&mut self, stmt: &Stmt);

}