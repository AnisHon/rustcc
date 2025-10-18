use crate::err::parser_error;
use crate::err::parser_error::{ParserError, ParserResult};
use crate::lex::types::token_kind::{Keyword, TokenKind};
use crate::parser::parser_core::Parser;
use crate::parser::types::ast::decl::{Decl, DeclGroup, DeclKind, EnumField, EnumFieldList, Initializer, InitializerList, StructOrUnion};
use crate::parser::types::common::{Ident, IdentList};
use crate::parser::types::decl_spec::*;
use crate::parser::types::declarator::*;
use crate::parser::types::sema::decl::decl_context::DeclContextKind;
use crate::types::span::{Pos, Span};
use std::rc::Rc;

macro_rules! dup_error {
    ($ele:expr, $context:expr) => {{
        let item = $ele.kind_str().to_owned();
        let kind = parser_error::ErrorKind::Duplicate { item, context: $context.to_owned() };
        ParserError::new(kind, $ele.span)
    }};
}

macro_rules! combine_error {
    ($ele:expr, $context:expr) => {{
        let prev = $ele.kind_str().to_owned();
        let kind = parser_error::ErrorKind::NonCombinable { prev , context: $context.to_owned() };
        ParserError::new(kind, $ele.span)
    }};
}

impl Parser {

    fn check_declarator(&self) -> bool {
        let kind = &self.stream.peek().kind;
        match kind {
            TokenKind::LParen
            | TokenKind::LBrace
            | TokenKind::LBracket
            | TokenKind::Ident(_) => true,
            _ => false,
        }
    }

    fn check_pointer(&self) -> bool {
        let kind = &self.stream.peek().kind;
        matches!(kind, TokenKind::Star)
    }

    pub(crate) fn parse_decl(&mut self) -> ParserResult<DeclGroup> {
        let lo = self.stream.span();

        let decl_spec = self.parse_decl_spec()?;
        let mut declarator = Declarator::new(decl_spec);
        self.parse_declarator(&mut declarator)?;

        self.parse_decl_after_declarator(lo, declarator)
    }

    pub(crate) fn parse_decl_after_declarator(
        &mut self,
        lo: Span,
        declarator: Declarator,
    ) -> ParserResult<DeclGroup> {
        let mut group = DeclGroup::default();
        self.parse_init_declarator_list(declarator, &mut group)?;
        let semi = self.expect(TokenKind::Semi)?.span.to_pos();

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);
        group.semi  = semi;
        group.span = span;

        Ok(group)
    }


    pub(crate) fn parse_decl_spec(&mut self) -> ParserResult<Rc<DeclSpec>> {
        const  CONTEXT: &str = "declaration specifier";

        let lo = self.stream.span();

        let mut storage: Option<StorageSpec> = None;
        let mut type_specs: Vec<TypeSpec> = Vec::new();
        let mut type_quals: [Option<TypeQual>; 3] = [None; 3];
        let mut func_spec: Option<FuncSpec> = None;


        loop {
            let token = self.stream.peek();
            if self.is_storage_spec(token) {
                // typedef extern static auto register
                let spec = self.parse_storage_spec()?;

                if let Some(storage) = &storage {
                    let error = if storage.kind == spec.kind {
                         dup_error!(spec, CONTEXT)
                    } else {
                        combine_error!(storage, CONTEXT)
                    };
                    self.send_error(error)
                }
                storage = Some(spec);
            } else if self.is_type_spec(token) {
                // 类型
                let spec = self.parse_type_spec()?;
                type_specs.push(spec);

            } else if self.is_type_qual(token) {
                // const restrict volatile
                let qual = self.parse_type_qual()?;
                let idx = qual.kind as usize;

                if type_quals[idx].is_some() {
                    let error = dup_error!(qual, CONTEXT);
                    self.send_error(error)
                }

                type_quals[idx] = Some(qual);
            } else if self.check_keyword(Keyword::Inline) {
                // inline
                let spec = self.parse_function_spec()?;

                if func_spec.is_some() {
                    let error = dup_error!(spec, CONTEXT);
                    self.send_error(error)
                }
                func_spec = Some(spec);
            } else {
                break
            };
        }


        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);


        let decl_spec = Rc::new(DeclSpec {
            storage,
            type_specs,
            type_quals,
            func_spec,
            span
        });
        Ok(decl_spec)
    }

    fn parse_storage_spec(&mut self) -> ParserResult<StorageSpec> {
        let token = self.stream.next();
        let storage_spec = StorageSpec::new(token);
        Ok(storage_spec)
    }

    fn parse_type_spec(&mut self) -> ParserResult<TypeSpec> {
        let token = self.stream.peek();
        let span = token.span;
        let spec = match &token.kind {
            TokenKind::Ident(_) => {
                let symbol = self.stream.next().kind.into_ident().unwrap();
                let ident = Ident { symbol, span };
                let decl = match self.sema.curr_decl.borrow().lookup_chain(symbol) {
                    Some(x) => x,
                    None => todo!()
                };
                let kind = TypeSpecKind::TypeName(ident, decl);
                TypeSpec { kind, span }
            }
            TokenKind::Keyword(kw) => match kw {
                Keyword::Struct => {
                    let spec = self.parse_struct_or_union_spec()?;
                    let kind = TypeSpecKind::Struct(spec);
                    TypeSpec { kind, span }
                }
                Keyword::Union => {
                    let spec = self.parse_struct_or_union_spec()?;
                    let kind = TypeSpecKind::Union(spec);
                    TypeSpec { kind, span }
                }
                Keyword::Enum => {
                    let spec = self.parse_enum_spec()?;
                    let kind = TypeSpecKind::Enum(spec);
                    TypeSpec { kind, span }
                }
                _ => TypeSpec::new(self.stream.next())
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
        let lo = self.stream.span();

        if self.check_pointer() {
            self.parse_pointer(declarator)?;
        }
        if self.check_declarator() {
            self.parse_direct_declarator(declarator)?;
        }

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);
        declarator.span = span;
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
                let type_qual = self.parse_type_qual_list_opt()?;
                // 是否是空括号[]
                let expr = match self.check(TokenKind::RBracket) {
                    true =>  None, // 空括号
                    false => Some(self.parse_assign_expr()?) // 非空解析为表达式
                };
                let r = self.expect(TokenKind::RBracket)?.span.to_pos();
                DeclaratorChunkKind::Array { l, type_qual, expr, r }
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
            let type_qual = match self.is_type_qual(self.stream.peek()) {
                true => self.parse_type_qual_list()?,
                false => [None; 3],
            };

            let hi = self.stream.prev_span();
            let span = Span::span(lo, hi);

            let kind = DeclaratorChunkKind::Pointer { star, type_qual };
            let chunk = DeclaratorChunk::new(kind, span);

            declarator.chunks.push(chunk);
        }

        Ok(())
    }

    fn parse_type_qual_list_opt(&mut self) -> ParserResult<Option<TypeQualType>> {
        if self.is_type_qual(self.stream.peek()) {
            self.parse_type_qual_list().map(|list| Some(list))
        } else {
            Ok(None)
        }
    }

    fn parse_type_qual_list(&mut self) -> ParserResult<TypeQualType> {
        let mut type_qual: [Option<TypeQual>; 3] = [None; 3];
        loop {
            if self.is_type_qual(self.stream.peek()) {
                let qual = TypeQual::new(self.stream.next());
                let idx = qual.kind as usize;

                if type_qual[idx].is_some() {
                    let error = dup_error!(qual, "Declaration Specifier");
                    self.send_error(error);
                }

                type_qual[idx] = Some(qual);
            } else {
                break;
            }
        }
        Ok(type_qual)
    }



    fn parse_init_declarator_list(&mut self, declarator: Declarator, group: &mut DeclGroup) -> ParserResult<()> {
        let decl_spec = Rc::clone(&declarator.decl_spec);

        let init = self.parse_init_declarator(Rc::clone(&decl_spec), Some(declarator))?;
        group.decls.push(init);

        while let Some(comma) = self.consume(TokenKind::Comma) {
            let init = self.parse_init_declarator(Rc::clone(&decl_spec), None)?;
            group.commas.push(comma.span.to_pos());
            group.decls.push(init);
        }
        Ok(())
    }

    ///
    /// # Arguments
    /// - `decl_spec`: DeclSpec引用
    /// - `declarator`: 传入None表示无Declarator
    fn parse_init_declarator(&mut self, decl_spec: Rc<DeclSpec>, declarator: Option<Declarator>) -> ParserResult<Rc<Decl>> {
        let lo = self.stream.span();

        // 解析declarator
        let declarator = match declarator {
            Some(x) => x,
            None => {
                let mut declarator = Declarator::new(decl_spec);
                self.parse_declarator(&mut declarator)?;
                declarator
            }
        };
        let mut eq: Option<Pos> = None;
        let mut init: Option<Initializer> = None;
        if let Some(assign_token) = self.consume(TokenKind::Assign) {
            // 解析initializer部分
            eq = Some(assign_token.span.to_pos());
            init = Some(self.parse_initializer()?);
        }

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let init_declarator = InitDeclarator { declarator, eq, init, span };
        let decl = self.sema.act_on_init_declarator(init_declarator)?;
        Ok(decl)
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
            list.commas.push(comma.span.to_pos());
            list.inits.push(init);
        }
        Ok(list)
    }

    fn parse_struct_or_union_spec(&mut self) -> ParserResult<Rc<Decl>> {
        // 进入struct上下文
        self.sema.decl_enter(DeclContextKind::Record);
        let lo = self.stream.span();
        
        // 消耗struct union关键字
        let kw = self.expect_keyword_pair(Keyword::Struct, Keyword::Union)?;
        let record_kind = StructOrUnion::new(kw);
        let name = self.consume_ident().map(Ident::new); // 尝试解析名字
        let mut body = None;
        // 尝试解析内部声明
        if let Some(lbrace) = self.consume(TokenKind::LBrace) { 
            let r = lbrace.span.to_pos(); 
            let group = self.parse_struct_decl_list()?;
            let l = self.expect(TokenKind::RBrace)?.span.to_pos();
            body = Some(StructSpecBody { r, groups: group, l })
        }

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        // 退出struct上下文
        let decl_context = self.sema.decl_exit();
        
        // 语义分析
        let spec = StructSpec { kind: record_kind, name, body, span  };
        
        let decl = self.sema.act_on_finish_record(spec)?;
        Ok(decl)
    }

    /// 结构体内部声明，不负责括号
    fn parse_struct_decl_list(&mut self) -> ParserResult<Vec<DeclGroup>> {
        let mut decls = Vec::new();

        if self.check(TokenKind::RBrace) {
            return Ok(decls);
        }

        loop {
            let group = self.parse_struct_decl()?;
            decls.push(group);
            if self.check(TokenKind::RBrace) {
                break;
            }
        }

        Ok(decls)
    }

    /// 结构体成员声明，包括结尾分号
    fn parse_struct_decl(&mut self) -> ParserResult<DeclGroup> {
        let lo = self.stream.span();

        let decl_spec = self.parse_decl_spec()?;
        let mut group = DeclGroup::default();
        self.parse_struct_declarator_list(&mut group, decl_spec)?;
        let semi = self.expect(TokenKind::Semi)?.span.to_pos();

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);
        group.semi = semi;
        group.span = span;

        Ok(group)
    }

    /// 结构体声明declarator列表形如 *a, **b, **c 
    fn parse_struct_declarator_list(&mut self, group: &mut DeclGroup, decl_spec: Rc<DeclSpec>) -> ParserResult<()> {

        // 构建declarator
        
        let decl = self.parse_struct_declarator(Rc::clone(&decl_spec))?; 
        group.decls.push(decl);

        while let Some(comma) = self.consume(TokenKind::Comma) {
            let comma = comma.span.to_pos();
            let decl = self.parse_struct_declarator(Rc::clone(&decl_spec))?;
            group.decls.push(decl);
            group.commas.push(comma);
        }

        Ok(())
    }
    
    ///
    fn parse_struct_declarator(&mut self, decl_spec: Rc<DeclSpec>) -> ParserResult<Rc<Decl>> {
        let mut declarator = Declarator::new(decl_spec);
        
        let lo = self.stream.span();
        
        let mut colon = None;
        let mut bit_field = None;

        if self.check_declarator() {
            self.parse_declarator(&mut declarator)?;
        }

        if let Some(colon_token) = self.consume(TokenKind::Colon) {
            colon = Some(colon_token.span.to_pos());
            bit_field = Some(self.parse_assign_expr()?);
        }
        
        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let struct_declarator = StructDeclarator { declarator, colon, bit_field, span };

        // 语义分析，获取类型
        let decl = self.sema.act_on_record_field(struct_declarator)?;
        Ok(decl)
    }


    fn parse_enum_spec(&mut self) -> ParserResult<Rc<Decl>> {
        // 准备枚举上下文
        self.sema.decl_enter(DeclContextKind::Enum);
        let lo = self.stream.span();
        
        let kw = self.expect_keyword(Keyword::Enum)?.span;

        // 检查是否合法
        if self.check_ident() || self.check(TokenKind::LBrace) {
            let kind = parser_error::ErrorKind::Expect { expect: "identifier or '{'".to_owned() };
            return Err(self.error_here(kind));
        }

        let name= self.consume_ident().map(Ident::new);
        let body;
        if let Some(lbrace) = self.consume(TokenKind::LBrace) {
            let l = lbrace.span.to_pos();
            let list = self.parse_enumerator_list()?;
            let r = self.expect(TokenKind::RBrace)?.span.to_pos();
            body = Some(EnumSpecBody { l, list, r });
        } else {
            // 出错
            let expect = "identifier or '{'".to_owned();
            let kind = parser_error::ErrorKind::Expect { expect };
            let error = self.error_here(kind);
            return Err(error);
        }

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);


        let spec = EnumSpec { enum_span: kw, name, body, span };
        // 完成并结束枚举上下文
        let decl = self.sema.act_on_finish_enum(spec)?;
        Ok(decl)
    }

    fn parse_enumerator_list(&mut self) -> ParserResult<EnumFieldList> {
        let lo = self.stream.span();

        let mut list = EnumFieldList::default();
        
        loop {
            let decl = self.parse_enumerator()?;
            list.decls.push(decl);

            if let Some(comma) = self.consume(TokenKind::Comma) { 
                list.commas.push(comma.span.to_pos());
            } else {
                break;
            }
        }
        
        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);
        list.span = span;

        Ok(list)
    }

    fn parse_enumerator(&mut self) -> ParserResult<EnumField> {
        let lo = self.stream.span();
        
        let ident = self.expect_ident()?;
        let name = Ident::new(ident);
        let mut eq = None;
        let mut expr = None;
        if let Some(assign_token) = self.consume(TokenKind::Assign) {
            eq = Some(assign_token.span.to_pos());
            expr = Some(self.parse_assign_expr()?);
        };

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);
        
        let field = EnumField { name, eq, expr, span };
        Ok(field)
    }

    /// 函数列表，不包含左右括号
    fn parse_parameter_list(&mut self) -> ParserResult<ParamList> {
        // 进入函数列表上下文
        self.sema.decl_enter(DeclContextKind::Param);
        
        let lo = self.stream.span();

        let mut params: Vec<Rc<Decl>> = Vec::new();
        let mut commas = Vec::new();
        let mut ellipsis = None;

        // 解析第一个参数声明
        let decl = self.parse_parameter_decl()?;
        params.push(decl);

        // 解析后续参数声明
        while let Some(comma) = self.consume(TokenKind::Comma) {
            commas.push(comma.span.to_pos());
            if let Some(token) = self.consume(TokenKind::Ellipsis) {
                ellipsis = Some(token.span);
                break
            }
            let decl = self.parse_parameter_decl()?;
            params.push(decl);
        }

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let decl_context = self.sema.decl_exit();
        let list = ParamList { params, commas, ellipsis, decl_context, span };

        Ok(list)
    }

    fn parse_parameter_decl(&mut self) -> ParserResult<Rc<Decl>> {
        let lo = self.stream.span();

        let decl_spec = self.parse_decl_spec()?;
        let mut declarator = Declarator::new(decl_spec);
        self.parse_declarator(&mut declarator)?;

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        declarator.span = span;
        let decl = self.sema.act_on_declarator(declarator)?;
        Ok(decl)
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
        let mut declarator = Declarator::new(decl_specs);
        if self.check_declarator() {
            self.parse_declarator(&mut declarator)?;
        };

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        declarator.span = span;

        todo!()
    }
}