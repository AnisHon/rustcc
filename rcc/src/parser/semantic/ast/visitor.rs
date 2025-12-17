use crate::parser::ast::decl::DeclKey;
use crate::parser::ast::exprs::ExprKey;
use crate::parser::ast::func::FuncDef;
use crate::parser::ast::stmt::StmtKey;
use crate::parser::semantic::ast::decl::DeclGroup;
use crate::parser::semantic::ast::func::{ExternalDecl, TranslationUnit};

pub trait Visitor {
    fn walk_translation_unit(&mut self, unit: &mut TranslationUnit) {
        for ext_decl in unit {
            self.walk_external_decl(ext_decl);
        }
    }

    fn walk_external_decl(&mut self, decl: &mut ExternalDecl) {
        match decl {
            ExternalDecl::FunctionDefinition(x) => self.walk_func_def(x),
            ExternalDecl::Declaration(x) => self.walk_decl_group(x)
        }
    }

    fn walk_func_def(&mut self, decl: &mut FuncDef) {
        self.visit_decl(decl.decl);
        self.visit_stmt(decl.body);
    }

    fn walk_decl_group(&mut self, decl_group: &DeclGroup) {
        for x in decl_group.decls.iter().cloned() {
            self.visit_decl(x);
        }
    }

    fn visit_decl(&mut self, decl: DeclKey);

    fn visit_expr(&mut self, expr: ExprKey);

    fn visit_stmt(&mut self, stmt: StmtKey);

}