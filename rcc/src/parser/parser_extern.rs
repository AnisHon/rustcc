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
        let start = self.peek().range;
        let spec = self.declaration_specifiers()?;
        if self.eat(&TokenKind::Semi).is_some() {
            let (context, linkage, storage_duration) =
                self.sema.declaration_properties(spec.storage, &spec.ty);
            return Ok(vec![ExternalDeclaration::Declaration(Declaration {
                id: self.sema.fresh_decl_id(),
                previous_declaration: None,
                context,
                linkage,
                storage_duration,
                name: None,
                ty: spec.ty,
                storage: spec.storage,
                function_specifiers: spec.function_specifiers,
                initializer: None,
                alignment: spec.alignment,
                range: start.join(self.previous().range),
            })]);
        }
        let node = self.declarator(false)?;
        let (name, mut ty, mut params) = self.apply_declarator(node, spec.ty.clone())?;
        self.sema.validate_type(&ty, start)?;
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
            let declaration_id = self.sema.fresh_decl_id();
            let (context, linkage, storage_duration) =
                self.sema.declaration_properties(spec.storage, &ty);
            let mut d = Declaration {
                id: declaration_id,
                previous_declaration: None,
                context,
                linkage,
                storage_duration,
                name: Some(name.clone()),
                ty: ty.clone(),
                storage: spec.storage,
                function_specifiers: spec.function_specifiers,
                initializer: None,
                alignment: spec.alignment,
                range: start,
            };
            self.sema.declare_function_definition(&mut d)?;
            let return_type = match &ty.kind {
                TypeKind::Function { return_type, .. } => (**return_type).clone(),
                _ => unreachable!(),
            };
            let body_context = self.sema.begin_function(&mut params, return_type);
            let body_result = self.compound_statement(false);
            let end_result = self.sema.end_function();
            let body = body_result?;
            end_result?;
            let span = start.join(body.range);
            return Ok(vec![ExternalDeclaration::Function(FunctionDefinition {
                id: declaration_id,
                previous_declaration: d.previous_declaration,
                context,
                body_context,
                linkage,
                name,
                ty,
                storage: spec.storage,
                function_specifiers: spec.function_specifiers,
                parameters: params,
                body,
                range: span,
            })]);
        }
        let mut decls = vec![];
        let mut first = self.finish_declaration(start, spec.clone(), name, ty)?;
        self.sema.declare(&mut first)?;
        decls.push(ExternalDeclaration::Declaration(first));
        while self.eat(&TokenKind::Comma).is_some() {
            let node = self.declarator(false)?;
            let (n, t, _) = self.apply_declarator(node, spec.ty.clone())?;
            let mut d = self.finish_declaration(start, spec.clone(), n, t)?;
            self.sema.declare(&mut d)?;
            decls.push(ExternalDeclaration::Declaration(d));
        }
        self.expect(&TokenKind::Semi)?;
        Ok(decls)
    }
}
