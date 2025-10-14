use crate::err::parser_error::ParserResult;
use crate::lex::types::token_kind::{Keyword, TokenKind};
use crate::parser::parser_core::Parser;
use crate::parser::types::ast::decl::{Decl, EnumDecl, Initializer};
use crate::parser::types::ast::expr::Parameter;
use crate::parser::types::common::Ident;
use crate::parser::types::sema::decl_chunk::{DeclChunk, DeclChunkKind, Declarator, IdentList, InitDeclarator, InitDeclaratorList, InitializerList, ParamDecl, PointerChunk};
use crate::parser::types::sema::decl_spec::{DeclSpec, FuncSpec, SpecQualList, StorageSpec, TypeQual, TypeSpec, TypeSpecKind};
use crate::types::span::Span;

impl Parser {


    pub(crate) fn parse_decl(&mut self) -> ParserResult<Decl> {
        let lo = self.stream.span();
        let decl_spec = self.parse_decl_spec()?;
        let declarator = self.parse_declarator()?;
        self.parse_decl_after_declarator(lo, decl_spec, declarator)
    }

    pub(crate) fn parse_decl_after_declarator(
        &mut self,
        lo: Span,
        decl_spec: DeclSpec,
        declarator: Declarator
    ) -> ParserResult<Decl> {
        let init_list = self.parse_init_declarator_list(declarator)?;
        let semi = self.expect(TokenKind::Semi)?;
        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);
        todo!()
    }

    pub(crate) fn parse_decl_spec(&mut self) -> ParserResult<DeclSpec> {
        let lo = self.stream.span();
        let mut decl_specs = DeclSpec::new();
        loop {
            let token = self.stream.peek();
            if self.is_storage_spec(token) {
                let spec = self.parse_storage_spec()?;
                decl_specs.storages.push(spec);
            } else if self.is_type_spec(token) {
                let spec = self.parse_type_spec()?;
                decl_specs.type_specs.push(spec);
            } else if self.is_type_qual(token) {
                let qual = self.parse_type_qual()?;
                decl_specs.type_quals.push(qual);
            } else if self.check_keyword(Keyword::Inline) {
                // inline
                let spec = self.parse_function_spec()?;
                decl_specs.func_specs.push(spec);
            } else {
                break
            };
        }
        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        decl_specs.span = span;

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
                    let kind = TypeSpecKind::Struct();
                    TypeSpec { kind, span: token.span }
                }
                Keyword::Union => {
                    let spec = self.parse_struct_or_union_spec()?;
                    let kind = TypeSpecKind::Union();
                    TypeSpec { kind, span: token.span }
                }
                Keyword::Enum => {
                    let spec = self.parse_enum_spec()?;
                    let kind = TypeSpecKind::Enum();
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

    pub(crate) fn parse_declarator(&mut self) -> ParserResult<Declarator> {
        let pointers = match self.check(TokenKind::Star) {
            true => Some(self.parse_pointer()?),
            false => None,
        };
        let direct_declarator = self.parse_direct_declarator();
        todo!()
    }

    fn parse_direct_declarator_fist(&mut self) -> ParserResult<DeclChunk> {
        let lo = self.stream.span();
        let kind = if let Some(ident) = self.consume_ident() {
            let ident = Ident::new(ident);
            DeclChunkKind::Ident(ident)
        } else if let Some(lparen) = self.consume(TokenKind::LParen) {
            let l = lparen.span;
            let declarator = self.parse_declarator()?;
            let r = self.expect(TokenKind::RParen)?.span;
            DeclChunkKind::Paren { l, declarator, r }
        } else {
            unreachable!()
        };
        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);
        let chunk = DeclChunk::new(kind, span);
        Ok(chunk)
    }

    fn parse_direct_declarator(&mut self) -> ParserResult<Vec<DeclChunk>> {
        let chunk = self.parse_direct_declarator_fist()?;
        let mut chunks = vec![chunk];

        loop {
            let lo = self.stream.span();
            let kind = if let Some(lbracket) = self.consume(TokenKind::LBracket) {
                let l = lbracket.span;
                let type_quals = self.parse_type_qual_list_opt()?;
                let expr = match self.check(TokenKind::RBracket) {
                    true => Some(self.parse_assign_expr()?),
                    false => None
                };
                let r = self.expect(TokenKind::RBracket)?.span;
                DeclChunkKind::Array { l, type_quals, expr, r }
            } else if let Some(lparen) = self.consume(TokenKind::LParen) {
                let l = lparen.span;
                let param = match self.check_ident() {
                    true => self.parse_parameter_type_list()?,
                    false => {
                        let idents = self.parse_ident_list()?;
                        ParamDecl::Idents(idents)
                    }
                };
                let expr = self.parse_assign_expr()?;
                let r = self.expect(TokenKind::RParen)?.span;
                DeclChunkKind::Function { l, param, r }
            } else {
                break;
            };
            let hi = self.stream.prev_span();
            let span = Span::span(lo, hi);
            let chunk = DeclChunk::new(kind, span);
            chunks.push(chunk)
        }

        Ok(chunks)
    }

    fn parse_pointer(&mut self) -> ParserResult<Vec<PointerChunk>> {

        let mut chunks = Vec::new();

        loop {
            let lo = self.stream.span();
            if self.consume(TokenKind::Star).is_none() {
                break;
            }
            let quals = match self.is_type_qual(self.stream.peek()) {
                true => self.parse_type_qual_list()?,
                false => Vec::new(),
            };
            let hi = self.stream.prev_span();
            let span = Span::span(lo, hi);
            let chunk = PointerChunk::new(quals, span);
            chunks.push(chunk);
        }



        todo!()
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
    fn parse_init_declarator(&mut self, declarator: Option<Declarator>) -> ParserResult<InitDeclarator> {
        let declarator = match declarator {
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
        Ok(InitDeclarator { declarator, eq, init })
    }

    fn parse_init_declarator_list(&mut self, declarator: Declarator) -> ParserResult<InitDeclaratorList> {
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

    fn parse_struct_or_union_spec(&mut self) -> ParserResult<()> {
        let kw = self.expect_keyword_pair(Keyword::Struct, Keyword::Union)?;
        if let Some(lbrace) = self.consume(TokenKind::LBrace) {
            let rbrace = self.expect(TokenKind::RBrace)?;
        } else if let Some(ident) = self.consume_ident() {

        } else {
            // return Err()
        }

        todo!()
    }

    fn parse_struct_decl_list(&mut self) -> ParserResult<Vec<()>> {
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

    fn parse_struct_decl(&mut self) -> ParserResult<()> {
        todo!()
    }

    fn parse_spec_qual_list(&mut self) -> ParserResult<SpecQualList> {
        let lo = self.stream.span();
        let mut specs = SpecQualList::new();
        loop {
            let token = self.stream.peek();
            if self.is_type_spec(token) {
                let spec = self.parse_type_spec()?;
                specs.type_specs.push(spec);
            } else if self.is_type_qual(token) {
                let qual = self.parse_type_qual()?;
                specs.type_quals.push(qual);
            } else {
                break
            };
        }
        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        specs.span = span;

        Ok(specs)
    }

    fn parse_enum_spec(&mut self) -> ParserResult<()> {
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

    fn parse_enumerator_list(&mut self) -> ParserResult<Vec<()>> {
        todo!()
    }

    fn parse_enumerator(&mut self) -> ParserResult<EnumDecl> {
        let ident = self.expect_ident()?;
        let assign;
        let expr;
        if let Some(assign_token) = self.consume(TokenKind::Assign) {
            assign = Some(assign_token);
            expr = Some(self.parse_expr()?);
        } else {
            assign = None;
            expr = None;
        }

        todo!()
    }



    fn parse_parameter_type_list(&mut self) -> ParserResult<ParamDecl> {
        let mut param_decl = self.parse_parameter_decl()?;
        if let Some(comma) = self.consume(TokenKind::Comma) {
            let ellipsis_span = self.expect(TokenKind::Ellipsis)?.span;
            match &mut param_decl {
                ParamDecl::Idents(_) => unreachable!(),
                ParamDecl::Params { commas, ellipsis, .. } => {
                    commas.push(comma.span);
                    *ellipsis = Some(ellipsis_span);
                }
            }
        }
        Ok(param_decl)
    }

    fn parse_parameter_decl(&mut self) -> ParserResult<ParamDecl> {
        todo!()
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
        // todo
        self.stream.next();
        Ok(())
    }

    fn parse_abstract_declarator(&mut self) -> ParserResult<()> {
        todo!()
    }

    fn parse_direct_abstract_declarator(&mut self) -> ParserResult<()> {
        todo!()
    }

}