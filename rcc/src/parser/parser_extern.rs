use super::parser_core::{PResult, Parser};
use crate::lex::token::{Keyword, TokenKind};
use crate::parser::ast::*;

impl Parser {
    pub(crate) fn external_declaration(&mut self) -> PResult<Vec<ExternalDeclaration>> {
        if self.kw(Keyword::StaticAssert) {
            return Ok(vec![ExternalDeclaration::StaticAssert(
                self.static_assert()?,
            )]);
        }
        let start = self.peek().span;
        let spec = self.declaration_specifiers()?;
        if self.eat(&TokenKind::Semi).is_some() {
            return Ok(vec![ExternalDeclaration::Declaration(Declaration {
                name: None,
                ty: spec.ty,
                storage: spec.storage,
                function_specifiers: spec.function_specifiers,
                initializer: None,
                alignment: spec.alignment,
                span: start.join(self.previous().span),
            })]);
        }
        let node = self.declarator(false)?;
        let (name, mut ty, mut params) = self.apply_declarator(node, spec.ty.clone())?;
        let old_style = matches!(
            &ty.kind,
            TypeKind::Function {
                has_prototype: false,
                ..
            }
        ) && !self.at(&TokenKind::LBrace)
            && self.is_declaration_start();
        if old_style {
            self.old_parameter_declarations(&mut params)?;
            if let TypeKind::Function {
                params: function_params,
                ..
            } = &mut ty.kind
            {
                *function_params = params.clone();
            }
        }
        if self.at(&TokenKind::LBrace) {
            let name = name.ok_or_else(|| self.err("function definition requires a name"))?;
            let TypeKind::Function { .. } = ty.kind else {
                return Err(self.err("only a function declarator may have a body"));
            };
            let d = Declaration {
                name: Some(name.clone()),
                ty: ty.clone(),
                storage: spec.storage,
                function_specifiers: spec.function_specifiers,
                initializer: None,
                alignment: spec.alignment,
                span: start,
            };
            self.sema.declare(&d)?;
            let return_type = match &ty.kind {
                TypeKind::Function { return_type, .. } => (**return_type).clone(),
                _ => unreachable!(),
            };
            self.sema.begin_function(&params, return_type);
            let body = self.compound_statement(false)?;
            self.sema.end_function();
            let span = start.join(body.span);
            return Ok(vec![ExternalDeclaration::Function(FunctionDefinition {
                name,
                ty,
                storage: spec.storage,
                function_specifiers: spec.function_specifiers,
                parameters: params,
                body,
                span,
            })]);
        }
        let mut decls = vec![];
        let first = self.finish_declaration(start, spec.clone(), name, ty)?;
        self.sema.declare(&first)?;
        decls.push(ExternalDeclaration::Declaration(first));
        while self.eat(&TokenKind::Comma).is_some() {
            let node = self.declarator(false)?;
            let (n, t, _) = self.apply_declarator(node, spec.ty.clone())?;
            let d = self.finish_declaration(start, spec.clone(), n, t)?;
            self.sema.declare(&d)?;
            decls.push(ExternalDeclaration::Declaration(d));
        }
        self.expect(&TokenKind::Semi)?;
        Ok(decls)
    }
}
