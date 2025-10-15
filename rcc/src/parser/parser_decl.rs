use crate::err::parser_error::ParserResult;
use crate::lex::types::token_kind::{Keyword, TokenKind};
use crate::parser::parser_core::Parser;
use crate::parser::types::ast::decl::{Decl, Initializer, InitializerList};
use crate::parser::types::common::Ident;
use crate::parser::types::sema::decl_chunk::*;
use crate::types::span::Span;

impl Parser {

    fn check_abstract_declarator(&self) -> bool {
        let kind = &self.stream.peek().kind;
        matches!(kind, TokenKind::LParen | TokenKind::LBrace | TokenKind::LBracket)
    }
    
    pub(crate) fn parse_decl(&mut self) -> ParserResult<Decl> {
        let lo = self.stream.span();
        let decl_spec = self.parse_decl_spec()?;
        let declarator = self.parse_declarator()?;
        self.parse_decl_after_declarator(lo, decl_spec, declarator)
    }

    pub(crate) fn parse_decl_after_declarator(
        &mut self,
        lo: Span,
        decl_specs: Vec<DeclSpec>,
        declarator: Vec<DeclaratorChunk>
    ) -> ParserResult<Decl> {
        let lo = self.stream.span();

        let init_list = self.parse_init_declarator_list(declarator)?;
        let semi = self.expect(TokenKind::Semi)?;

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        todo!()
    }

    pub(crate) fn parse_decl_spec(&mut self) -> ParserResult<Vec<DeclSpec>> {
        let mut decl_specs = Vec::new();
        loop {
            let token = self.stream.peek();
            let decl_spec = if self.is_storage_spec(token) {
                let spec = self.parse_storage_spec()?;
                DeclSpec::Storage(spec)
            } else if self.is_type_spec(token) {
                let spec = self.parse_type_spec()?;
                DeclSpec::TypeSpec(spec)
            } else if self.is_type_qual(token) {
                let qual = self.parse_type_qual()?;
                DeclSpec::TypeQual(qual)
            } else if self.check_keyword(Keyword::Inline) {
                // inline
                let spec = self.parse_function_spec()?;
                DeclSpec::FuncSpec(spec)
            } else {
                break
            };
            decl_specs.push(decl_spec);
        }

        Ok(decl_specs)
    }

    fn parse_storage_spec(&mut self) -> ParserResult<StorageSpec> {
        let token = self.stream.next();
        let storage_spec = StorageSpec::new(token);
        Ok(storage_spec)
    }

    fn parse_type_spec(&mut self) -> ParserResult<TypeSpec> {
        let token = self.stream.next();
        let spec = match token.kind {
            TokenKind::Ident(symbol) => {
                let ident = Ident { symbol, span: token.span };
                let kind = TypeSpecKind::Typedef(ident);
                TypeSpec { kind, span: token.span }
            }
            TokenKind::Keyword(kw) => match kw {
                Keyword::Struct => {
                    let spec = self.parse_struct_or_union_spec()?;
                    let kind = TypeSpecKind::Struct(spec);
                    TypeSpec { kind, span: token.span }
                }
                Keyword::Union => {
                    let spec = self.parse_struct_or_union_spec()?;
                    let kind = TypeSpecKind::Union(spec);
                    TypeSpec { kind, span: token.span }
                }
                Keyword::Enum => {
                    let spec = self.parse_enum_spec()?;
                    let kind = TypeSpecKind::Enum(spec);
                    TypeSpec { kind, span: token.span }
                }
                _ => TypeSpec::new(token)
            },
            _ => unreachable!()
        };
        Ok(spec)
    }

    fn parse_type_qual(&mut self) -> ParserResult<TypeQual> {
        let token = self.stream.next();
        let type_qual = TypeQual::new(token);
        Ok(type_qual)
    }

    fn parse_function_spec(&mut self) -> ParserResult<FuncSpec> {
        let inline = self.stream.next();
        let func_spec  = FuncSpec::new(inline);
        Ok(func_spec)
    }

    pub(crate) fn parse_declarator(&mut self) -> ParserResult<Vec<DeclaratorChunk>> {
        let pointers = match self.check(TokenKind::Star) {
            true => Some(self.parse_pointer()?),
            false => None,
        };
        let chunks = self.parse_direct_declarator()?;
        todo!()
    }

    fn parse_direct_declarator_fist(&mut self) -> ParserResult<DeclaratorChunk> {
        let lo = self.stream.span();

        let kind = if let Some(ident) = self.consume_ident() {
            let ident = Ident::new(ident);
            DeclaratorChunkKind::Ident(ident)
        } else if let Some(lparen) = self.consume(TokenKind::LParen) {
            let l = lparen.span;
            let declarator = self.parse_declarator()?;
            let r = self.expect(TokenKind::RParen)?.span;
            DeclaratorChunkKind::Paren { l, declarator, r }
        } else {
            unreachable!()
        };

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let chunk = DeclaratorChunk::new(kind, span);
        Ok(chunk)
    }

    fn parse_direct_declarator(&mut self) -> ParserResult<Vec<DeclaratorChunk>> {
        let chunk = self.parse_direct_declarator_fist()?;
        let mut chunks = vec![chunk];

        loop {
            let lo = self.stream.span();

            let kind = if let Some(lbracket) = self.consume(TokenKind::LBracket) {
                // array []
                let l = lbracket.span;
                let type_quals = self.parse_type_qual_list_opt()?;
                let expr = match self.check(TokenKind::RBracket) {
                    true => Some(self.parse_assign_expr()?),
                    false => None
                };
                let r = self.expect(TokenKind::RBracket)?.span;
                DeclaratorChunkKind::Array { l, type_quals, expr, r }
            } else if let Some(lparen) = self.consume(TokenKind::LParen) {
                // func ()
                let l = lparen.span;
                let param = match self.check_ident() {
                    true => {
                        let list = self.parse_parameter_type_list()?;
                        ParamDecl::Params(list)
                    },
                    false => {
                        let idents = self.parse_ident_list()?;
                        ParamDecl::Idents(idents)
                    }
                };
                let expr = self.parse_assign_expr()?;
                let r = self.expect(TokenKind::RParen)?.span;
                DeclaratorChunkKind::Function { l, param, r }
            } else {
                break;
            };

            let hi = self.stream.prev_span();
            let span = Span::span(lo, hi);

            let chunk = DeclaratorChunk::new(kind, span);
            chunks.push(chunk)
        }

        Ok(chunks)
    }

    fn parse_pointer(&mut self) -> ParserResult<Vec<DeclaratorChunk>> {
        let mut chunks = Vec::new();

        loop {
            let lo = self.stream.span();

            let star = match self.consume(TokenKind::Star) {
                Some(x) => x.span,
                None => break
            };
            let type_quals = match self.is_type_qual(self.stream.peek()) {
                true => self.parse_type_qual_list()?,
                false => Vec::new(),
            };

            let hi = self.stream.prev_span();
            let span = Span::span(lo, hi);

            let kind = DeclaratorChunkKind::Pointer { star, type_quals };
            let chunk = DeclaratorChunk::new(kind, span);
            chunks.push(chunk);
        }

        Ok(chunks)
    }

    fn parse_type_qual_list_opt(&mut self) -> ParserResult<Option<Vec<TypeQual>>> {
        if self.is_type_qual(self.stream.peek()) {
            self.parse_type_qual_list().map(|list| Some(list))
        } else {
            Ok(None)
        }
    }

    fn parse_type_qual_list(&mut self) -> ParserResult<Vec<TypeQual>> {
        let mut list = Vec::new();
        loop {
            if self.is_type_qual(self.stream.peek()) {
                let qual = TypeQual::new(self.stream.next());
                list.push(qual);
            } else {
                break;
            }
        }
        Ok(list)
    }

    /// 已经预先解析一个declarator了
    fn parse_init_declarator(&mut self, declarator: Option<Vec<DeclaratorChunk>>) -> ParserResult<InitDeclarator> {
        let chunks = match declarator {
            None => self.parse_declarator()?,
            Some(x) => x
        };
        let eq;
        let init;
        if let Some(assign_token) = self.consume(TokenKind::Assign) {
            eq = Some(assign_token.span);
            init = Some(self.parse_initializer()?);
        } else {
            eq = None;
            init = None;
        }
        let init_declarator = InitDeclarator { chunks, eq, init };
        Ok(init_declarator)
    }

    fn parse_init_declarator_list(&mut self, declarator: Vec<DeclaratorChunk>) -> ParserResult<InitDeclaratorList> {
        let mut list = InitDeclaratorList::new();
        let init = self.parse_init_declarator(Some(declarator))?;
        list.inits.push(init);

        while let Some(comma) = self.consume(TokenKind::Comma) {
            let init = self.parse_init_declarator(None)?;
            list.commas.push(comma.span);
            list.inits.push(init);
        }
        Ok(list)
    }

    fn parse_initializer(&mut self) -> ParserResult<Initializer> {
        let init = if let Some(lparen) = self.consume(TokenKind::LParen) {
            let l = lparen.span;
            let inits = self.parse_initializer_list()?;
            let r = self.expect(TokenKind::RParen)?.span;
            Initializer::InitList { l, inits, r }
        } else {
            let expr = self.parse_assign_expr()?;
            Initializer::Expr(expr)
        };
        Ok(init)
    }

    fn parse_initializer_list(&mut self) -> ParserResult<InitializerList> {
        let mut list = InitializerList::new();
        let init = self.parse_initializer()?;
        list.inits.push(init);

        while let Some(comma) = self.consume(TokenKind::Comma) {
            if self.check(TokenKind::RParen) {
                break;
            }
            let init = self.parse_initializer()?;
            list.commas.push(comma.span);
            list.inits.push(init);
        }
        Ok(list)
    }

    fn parse_struct_or_union_spec(&mut self) -> ParserResult<StructSpec> {
        let lo = self.stream.span();
        
        let kw = self.expect_keyword_pair(Keyword::Struct, Keyword::Union)?;
        let name = self.consume_ident().map(Ident::new);
        let mut l = None;
        let mut var_decls = None;
        let mut r = None;
        if let Some(lbrace) = self.consume(TokenKind::LBrace) {
            r = Some(lbrace.span);
            var_decls = Some(self.parse_struct_decl_list()?);
            l = Some(self.expect(TokenKind::RBrace)?.span);
        } else {
            // todo
            unreachable!("todo")
        }

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let spec = StructSpec { struct_span: kw.span, name, l, var_decls, r, span };
        Ok(spec)
    }

    fn parse_struct_decl_list(&mut self) -> ParserResult<Vec<StructVar>> {
        let mut decls = Vec::new();

        if self.check(TokenKind::RBrace) {
            return Ok(decls);
        }

        loop {
            let decl = self.parse_struct_decl()?;
            decls.push(decl);
            if self.check(TokenKind::RBrace) {
                break;
            }
        }

        Ok(decls)
    }

    fn parse_struct_decl(&mut self) -> ParserResult<StructVar> {
        let lo = self.stream.span();

        let decl_spec = self.parse_decl_spec()?;
        let list = self.parse_struct_declarator_list()?;

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let var_decl = StructVar { decl_spec, list, span };
        Ok(var_decl)
    }

    fn parse_struct_declarator_list(&mut self) -> ParserResult<StructDeclaratorList> {
        let lo = self.stream.span();
        let mut declarators = Vec::new();
        let mut commas = Vec::new();

        let struct_declarator = self.parse_struct_declarator()?;
        declarators.push(struct_declarator);

        while let Some(comma) = self.consume(TokenKind::Comma) {
            let comma = comma.span;
            let struct_declarator = self.parse_struct_declarator()?;
            declarators.push(struct_declarator);
            commas.push(comma);
        }

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let list = StructDeclaratorList { declarators, commas, span };
        Ok(list)
    }

    fn parse_struct_declarator(&mut self) -> ParserResult<StructDeclarator> {
        let lo = self.stream.span();
        
        let mut chunks = None;
        let mut colon = None;
        let mut bit_field = None;
        
        if let Some(colon_token) = self.consume(TokenKind::Colon) {
            colon = Some(colon_token.span);
            bit_field = Some(self.parse_assign_expr()?);
        } else {
            chunks = Some(self.parse_declarator()?);
            if let Some(colon_token) = self.consume(TokenKind::Colon) {
                colon = Some(colon_token.span);
                bit_field = Some(self.parse_assign_expr()?);
            }
        }
        
        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let struct_declarator = StructDeclarator { chunks, colon, bit_field, span };
        Ok(struct_declarator)
    }


    fn parse_enum_spec(&mut self) -> ParserResult<EnumSpec> {
        let kw = self.expect_keyword(Keyword::Enum)?;
        if let Some(lbrace) = self.consume(TokenKind::LBrace) {
            let enumerator = self.parse_enumerator_list();
            let rbrace = self.expect(TokenKind::RBrace)?;
        } else if let Some(ident) = self.consume_ident() {

        } else {
            // return Err()
        }
        todo!()
    }

    fn parse_enumerator_list(&mut self) -> ParserResult<EnumeratorList> {
        let lo = self.stream.span();
        let mut decls = Vec::new();
        let mut commas = Vec::new();

        loop {
            let decl = self.parse_enumerator()?;
            decls.push(decl);

            if let Some(comma) = self.consume(TokenKind::Comma) { 
                commas.push(comma.span);
            } else {
                break;
            }
        }
        
        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let list = EnumeratorList { decls, commas, span };
        Ok(list)
    }

    fn parse_enumerator(&mut self) -> ParserResult<Enumerator> {
        let lo = self.stream.span();
        
        let ident = self.expect_ident()?;
        let ident = Ident::new(ident);
        let mut eq = None;
        let mut expr = None;;
        if let Some(assign_token) = self.consume(TokenKind::Assign) {
            eq = Some(assign_token.span);
            expr = Some(self.parse_expr()?);
        };

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let decl = Enumerator { ident, eq, expr, span };
        Ok(decl)
    }



    fn parse_parameter_type_list(&mut self) -> ParserResult<ParamVarDeclList> {
        let mut param_decl = self.parse_parameter_list()?;
        Ok(param_decl)
    }

    fn parse_parameter_list(&mut self) -> ParserResult<ParamVarDeclList> {
        let lo = self.stream.span();

        let mut params = Vec::new();
        let mut commas = Vec::new();
        let mut ellipsis = None;
        let param_decl = self.parse_parameter_decl()?;
        params.push(param_decl);

        while let Some(comma) = self.consume(TokenKind::Comma) {
            commas.push(comma.span);
            if let Some(token) = self.consume(TokenKind::Ellipsis) {
                ellipsis = Some(token.span);
                break
            }

            let param_decl = self.parse_parameter_decl()?;
            params.push(param_decl);
        }

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let list = ParamVarDeclList { params, commas, ellipsis, span };
        Ok(list)
    }

    fn parse_parameter_decl(&mut self) -> ParserResult<Declarator> {
        let lo = self.stream.span();
        let decl_specs = self.parse_decl_spec()?;
        let chunks = self.parse_declarator()?;
        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let declarator = Declarator::new(decl_specs, chunks, span);
        Ok(declarator)
    }

    fn parse_ident_list(&mut self) -> ParserResult<IdentList> {
        let mut list = IdentList::new();
        let ident = self.expect_ident()?;
        let ident = Ident::new(ident);
        list.idents.push(ident);

        while let Some(comma) = self.consume(TokenKind::Comma) {
            let ident = self.expect_ident()?;
            let ident = Ident::new(ident);
            list.idents.push(ident);
            list.commas.push(comma.span);
        }

        Ok(list)
    }

    pub(crate) fn parse_type_name(&mut self) -> ParserResult<()> {
        let lo = self.stream.span();
        
        let decl_specs = self.parse_decl_spec()?;
        let chunks = match self.check_abstract_declarator() {
            true => self.parse_declarator()?,
            false => Vec::new(),
        };
        
        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);
        
        let declarator = Declarator::new(decl_specs, chunks, span);
        
        todo!()
    }
}