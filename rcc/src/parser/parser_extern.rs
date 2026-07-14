use crate::errors::parser::ParserResult;
use crate::parser::parser_core::Parser;
use crate::types::lex::token_kind::TokenKind;
use crate::types::parser::ast::decls::decl::DeclGroup;
use crate::types::parser::ast::func::{ExternalDecl, FuncDecl, FuncDef, TranslationUnit};
use crate::types::parser::ast::stmt::Stmt;
use crate::types::parser::declarator::DeclPrefix;
use crate::types::span::Span;

impl Parser<'_> {
    fn check_decl_spec(&self) -> bool {
        let token = self.stream.peek();
        self.is_type_spec(token)
            || Self::is_type_qual(token)
            || self.is_spec_qual(token)
            || Self::is_storage_spec(token)
            || self.is_func_spec(token)
    }

    pub(crate) fn parse_translation_unit(&mut self) -> ParserResult<TranslationUnit> {
        let mut translation_unit = TranslationUnit::new();

        // 进入 File 作用域

        while !self.check(TokenKind::Eof) {
            self.parse_external_decl(&mut translation_unit)?;
        }

        // 处理暂定定义
        todo!();
        // 退出 File 作用域
        Ok(translation_unit)
    }

    fn parse_external_decl(&mut self, translation_unit: &mut TranslationUnit) -> ParserResult<()> {
        // 解析前缀
        let prefix = self.parse_decl_prefix()?;

        // 函数声明后 可能接 `decl_spec`(K&R) `{` 而且 declarator 一定不为空
        let is_func = (self.check_decl_spec() || self.check(TokenKind::LBrace))
            && prefix.declarator.is_some();
        // todo 这里可能要复杂一些，检查 declaration 还是 function def, 可以搭配declarator
        let external_decl = if is_func {
            let def = self.parse_function_def(prefix)?;
            ExternalDecl::FunctionDefinition(def)
        } else {
            // 声明
            let group = self.parse_decl_after_declarator(prefix)?;
            ExternalDecl::Declaration(group)
        };
        translation_unit.push(external_decl);
        Ok(())
    }

    fn parse_function_def(&mut self, prefix: DeclPrefix) -> ParserResult<FuncDef> {
        debug_assert!(
            prefix.declarator.is_some(),
            "function declarator never be none"
        );
        // 进入参数作用域

        // KR函数的参数
        let decl_list = match self.check_decl_spec() {
            true => Some(self.parse_decl_list()?),
            false => None,
        };

        let hi = self.stream.prev_span();
        let span = Span::span(prefix.lo, hi);

        let func_decl = FuncDecl {
            declarator: prefix.declarator.expect("impossible"),
            decl_list,
            span,
        };

        // 函数声明
        let decl = Sema::act_on_func_decl(self.ctx, func_decl)?;

        // compound stmt会调用exit_decl
        let kind = self.parse_compound_stmt(false)?;

        let hi = self.stream.prev_span();
        let span = Span::span(prefix.lo, hi);

        let body = Stmt::new_key(self.ctx, kind, span);
        let def = FuncDef { decl, body, span };

        Ok(def)
    }

    fn parse_decl_list(&mut self) -> ParserResult<Vec<DeclGroup>> {
        let mut list = Vec::new();
        loop {
            if self.check(TokenKind::LBrace) {
                break;
            }
            let group = self.parse_decl()?;
            list.push(group)
        }
        Ok(list)
    }
}
