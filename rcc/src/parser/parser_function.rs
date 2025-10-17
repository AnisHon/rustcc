use crate::err::parser_error::ParserResult;
use crate::lex::types::token_kind::TokenKind;
use crate::parser::parser_core::Parser;
use crate::parser::types::ast::decl::{DeclGroup};
use crate::parser::types::ast::func::{ExternalDecl, FuncDef, TranslationUnit};
use crate::parser::types::ast::stmt::Stmt;
use crate::parser::types::declarator::Declarator;
use crate::parser::types::sema::decl::decl_context::DeclContextKind;
use crate::types::span::Span;

impl Parser {
    fn check_decl_spec(&self) -> bool {
        let token = self.stream.peek();
        self.is_type_spec(token) 
            || self.is_type_qual(token) 
            || self.is_spec_qual(token)
            || self.is_storage_spec(token)
            || self.is_func_spec(token)
    }
    
    pub(crate) fn parse_translation_unit(&mut self) -> ParserResult<TranslationUnit>{
        let mut translation_unit = TranslationUnit::new();
        while !self.check(TokenKind::Eof) {
            self.parse_external_decl(&mut translation_unit)?;
        }
        Ok(translation_unit)
    }
    
    fn parse_external_decl(&mut self, translation_unit: &mut TranslationUnit) -> ParserResult<()> {
        let lo = self.stream.span();
        let decl_spec = self.parse_decl_spec()?;
        let mut declarator = Declarator::new(decl_spec);
        self.parse_declarator(&mut declarator)?;
        
        let external_decl = if self.check_decl_spec() || self.check(TokenKind::LBrace) {
            // 进入函数体上下文
            self.sema.decl_enter(DeclContextKind::Block);
            let func_decl = self.sema.act_on_declarator(declarator)?;
            // 函数定义
            let decl_list = match self.check_decl_spec() { 
                true => self.parse_decl_list()?,
                false => Vec::new()
            };
            
            let lo = self.stream.span();
            let kind = self.parse_compound_stmt(false)?;
            let hi = self.stream.prev_span();
            let span = Span::span(lo, hi);

            let body = Stmt::new_box(kind, span);
            let func_def = FuncDef { func_decl, decl_list, body, span };
            
            // 退出函数体上下文
            self.sema.decl_exit();
            
            ExternalDecl::FunctionDefinition(func_def)
        } else {
            // 声明
            let group = self.parse_decl_after_declarator(lo, declarator)?;
            ExternalDecl::Declaration(group)
        };
        translation_unit.push(external_decl);
        Ok(())
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