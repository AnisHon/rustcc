use std::rc::Rc;
use crate::err::parser_error;
use crate::err::parser_error::ParserResult;
use crate::lex::types::token_kind::{Keyword, TokenKind};
use crate::parser::parser_core::Parser;
use crate::parser::types::ast::decl::{Decl, Initializer, InitializerList};
use crate::parser::types::common::{Ident, IdentList};
use crate::parser::types::declarator::*;
use crate::parser::types::decl_spec::*;
use crate::types::span::{Pos, Span};

impl Parser {

    fn check_abstract_declarator(&self) -> bool {
        let kind = &self.stream.peek().kind;
        matches!(kind, TokenKind::LParen | TokenKind::LBrace | TokenKind::LBracket)
    }

    fn check_pointer(&self) -> bool {
        let kind = &self.stream.peek().kind;
        matches!(kind, TokenKind::Star)
    }

    fn check_direct_declarator(&self) -> bool {
        let kind = &self.stream.peek().kind;
        matches!(kind, TokenKind::LParen | TokenKind::Ident(_))
    }

    pub(crate) fn parse_decl(&mut self) -> ParserResult<Decl> {
        let lo = self.stream.span();

        let decl_spec = Rc::new(self.parse_decl_spec()?);
        let mut declarator = Declarator::new(decl_spec);
        self.parse_declarator(&mut declarator)?;

        self.parse_decl_after_declarator(lo, declarator)
    }

    pub(crate) fn parse_decl_after_declarator(
        &mut self,
        lo: Span,
        mut declarator: Declarator,
    ) -> ParserResult<Decl> {
        
        let init_list = self.parse_init_declarator_list(&mut declarator)?;
        let semi = self.expect(TokenKind::Semi)?.span.to_pos();

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        // println!("{:#?}", decl_specs);
        println!("{:#?}", init_list);
        todo!()
    }

    pub(crate) fn parse_decl_spec(&mut self) -> ParserResult<DeclSpec> {
        let lo = self.stream.span();

        let mut storage = None;
        let mut type_spec = None;
        let mut type_quals: [Option<TypeQual>; 3] = [None; 3];
        let mut func_spec = None;
        loop {
            let token = self.stream.peek();
            if self.is_storage_spec(token) {
                let spec = self.parse_storage_spec()?;
                if storage.is_some() {
                    todo!()
                }
                storage = Some(spec);
            } else if self.is_type_spec(token) {
                let spec = self.parse_type_spec()?;
                if type_spec.is_some() {
                    todo!()
                }
                type_spec = Some(spec);
            } else if self.is_type_qual(token) {
                let qual = self.parse_type_qual()?;
                let idx = qual.kind as usize;
                if type_quals[idx].is_some() {
                    todo!()
                }
                type_quals[idx] = Some(qual);
            } else if self.check_keyword(Keyword::Inline) {
                // inline
                let spec = self.parse_function_spec()?;
                if func_spec.is_some() {
                    todo!()
                }
                func_spec = Some(spec);
            } else {
                break
            };
        }

        if type_spec.is_none() {
            todo!()
        }

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let storage = storage.unwrap_or_else(|| todo!());
        let type_spec = type_spec.unwrap();

        let decl_spec = DeclSpec { storage, type_spec, type_quals, func_spec, span };
        Ok(decl_spec)
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

    /// 兼容abstract_declarator
    pub(crate) fn parse_declarator(&mut self, declarator: &mut Declarator) -> ParserResult<()> {
        if self.check_pointer() {
            self.parse_pointer(declarator)?;
        }
        if self.check_abstract_declarator() {
            self.parse_direct_declarator(declarator)?;
        }
        Ok(())
    }

    fn parse_direct_declarator_first(&mut self, declarator: &mut Declarator) -> ParserResult<()> {
        let lo = self.stream.span();

        let kind = if let Some(ident) = self.consume_ident() {
            let ident = Ident::new(ident);
            declarator.name = Some(ident);
            return Ok(());
        } else if let Some(lparen) = self.consume(TokenKind::LParen) {
            let l = lparen.span.to_pos();
            self.parse_declarator(declarator)?;
            let r = self.expect(TokenKind::RParen)?.span.to_pos();

            DeclaratorChunkKind::Paren { l, r }
        } else {
            println!("{:?}", self.stream.next());
            println!("{:?}", self.stream.next());
            println!("{:?}", self.stream.next());
            unreachable!()
        };

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let chunk = DeclaratorChunk::new(kind, span);
        declarator.chunks.push(chunk);

        Ok(())
    }

    fn parse_direct_declarator(&mut self, declarator: &mut Declarator) -> ParserResult<()> {
        self.parse_direct_declarator_first(declarator)?;

        loop {
            let lo = self.stream.span();

            let kind = if let Some(lbracket) = self.consume(TokenKind::LBracket) {
                // array []
                let l = lbracket.span.to_pos();
                let type_quals = self.parse_type_qual_list_opt()?;
                // 是否是空括号[]
                let expr = match self.check(TokenKind::RBracket) {
                    true =>  None, // 空括号
                    false => Some(self.parse_assign_expr()?) // 非空解析为表达式
                };
                let r = self.expect(TokenKind::RBracket)?.span.to_pos();
                DeclaratorChunkKind::Array { l, type_quals, expr, r }
            } else if let Some(lparen) = self.consume(TokenKind::LParen) {
                // func ()
                let l = lparen.span.to_pos();
                // 是否是K&R那种定义
                let param = match self.check_ident() {
                    true => {
                        let idents = self.parse_ident_list()?;
                        ParamDecl::Idents(idents)
                    },
                    false => {
                        let list = self.parse_parameter_list()?;
                        ParamDecl::Params(list)
                    }
                };
                let r = self.expect(TokenKind::RParen)?.span.to_pos();
                DeclaratorChunkKind::Function { l, param, r }
            } else {
                break;
            };

            let hi = self.stream.prev_span();
            let span = Span::span(lo, hi);

            let chunk = DeclaratorChunk::new(kind, span);
            declarator.chunks.push(chunk)
        }

        Ok(())
    }

    fn parse_pointer(&mut self, declarator: &mut Declarator) -> ParserResult<()> {

        loop {
            let lo = self.stream.span();

            let star = match self.consume(TokenKind::Star) {
                Some(x) => x.span.to_pos(),
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

            declarator.chunks.push(chunk);
        }

        Ok(())
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
    fn parse_init_declarator(&mut self, declarator: &mut Declarator, has_declarator: bool) -> ParserResult<InitDeclarator> {
        let chunks = match has_declarator {
            true => self.parse_declarator(declarator)?,
            false => declarator.chunks.pop().unwrap()
        };
        let eq;
        let init;
        if let Some(assign_token) = self.consume(TokenKind::Assign) {
            eq = Some(assign_token.span.to_pos());
            init = Some(self.parse_initializer()?);
        } else {
            eq = None;
            init = None;
        }
        let init_declarator = InitDeclarator { chunks, eq, init };
        Ok(init_declarator)
    }

    fn parse_init_declarator_list(&mut self, declarator: &mut Declarator) -> ParserResult<InitDeclaratorList> {
        let mut list = InitDeclaratorList::new();
        let init = self.parse_init_declarator(declarator)?;
        list.inits.push(init);

        while let Some(comma) = self.consume(TokenKind::Comma) {
            let init = self.parse_init_declarator(declarator)?;
            list.commas.push(comma.span.to_pos());
            list.inits.push(init);
        }
        Ok(list)
    }

    fn parse_initializer(&mut self) -> ParserResult<Initializer> {
        let init = if let Some(lparen) = self.consume(TokenKind::LParen) {
            let l = lparen.span.to_pos();
            let inits = self.parse_initializer_list()?;
            let r = self.expect(TokenKind::RParen)?.span.to_pos();
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
            r = Some(lbrace.span.to_pos());
            var_decls = Some(self.parse_struct_decl_list()?);
            l = Some(self.expect(TokenKind::RBrace)?.span.to_pos());
        } else {
            // 出错
            let expect = "identifier or '{'".to_owned();
            let kind = parser_error::ErrorKind::Expect { expect };
            let error = self.error_here(kind);
            return Err(error);
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
            let comma = comma.span.to_pos();
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
            colon = Some(colon_token.span.to_pos());
            bit_field = Some(self.parse_assign_expr()?);
        } else {
            chunks = Some(self.parse_declarator()?);
            if let Some(colon_token) = self.consume(TokenKind::Colon) {
                colon = Some(colon_token.span.to_pos());
                bit_field = Some(self.parse_assign_expr()?);
            }
        }
        
        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let struct_declarator = StructDeclarator { chunks, colon, bit_field, span };
        Ok(struct_declarator)
    }


    fn parse_enum_spec(&mut self) -> ParserResult<EnumSpec> {
        let lo = self.stream.span();

        let kw = self.expect_keyword(Keyword::Enum)?;
        let name= self.consume_ident().map(Ident::new);
        let mut l: Option<Pos> = None;
        let mut enumerators = None;
        let mut r: Option<Pos> = None;
        if let Some(lbrace) = self.consume(TokenKind::LBrace) {
            l = Some(lbrace.span.to_pos());
            enumerators = Some(self.parse_enumerator_list()?);
            r = Some(self.expect(TokenKind::RBrace)?.span.to_pos());
        } else {
            // 出错
            let expect = "identifier or '{'".to_owned();
            let kind = parser_error::ErrorKind::Expect { expect };
            let error = self.error_here(kind);
            return Err(error);
        }

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let spec = EnumSpec { enum_span: kw.span, name, l, enumerators, r, span };
        Ok(spec)
    }

    fn parse_enumerator_list(&mut self) -> ParserResult<EnumeratorList> {
        let lo = self.stream.span();
        let mut decls = Vec::new();
        let mut commas = Vec::new();

        loop {
            let decl = self.parse_enumerator()?;
            decls.push(decl);

            if let Some(comma) = self.consume(TokenKind::Comma) { 
                commas.push(comma.span.to_pos());
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
            eq = Some(assign_token.span.to_pos());
            expr = Some(self.parse_assign_expr()?);
        };

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let decl = Enumerator { ident, eq, expr, span };
        Ok(decl)
    }

    fn parse_parameter_list(&mut self) -> ParserResult<ParamVarDeclList> {
        let lo = self.stream.span();

        let mut params = Vec::new();
        let mut commas = Vec::new();
        let mut ellipsis = None;
        let param_decl = self.parse_parameter_decl()?;
        params.push(param_decl);

        while let Some(comma) = self.consume(TokenKind::Comma) {
            commas.push(comma.span.to_pos());
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
            list.commas.push(comma.span.to_pos());
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