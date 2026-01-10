use std::rc::Rc;

use crate::constant::str::EXPECT_IDENT_OR_LB;
use crate::types::parser::decl_spec::{EnumSuffix, RecordSuffix, TypeQualKind};
use crate::types::parser::declarator::DeclPrefix;
use crate::{
    constant::str::DECL_SPEC,
    err::parser_error::{self, ParserError, ParserResult},
    types::span::Span,
};
use crate::types::parser::ast::decls::decl::{DeclGroup, InitializerList};
use crate::types::parser::ast::decls::initializer::Initializer;
use crate::parser::parser_core::Parser;
use crate::parser::sema::decl::record::insert_record_decl;
use crate::parser::sema::decl::decl_spec::DeclSpecBuilder;
use crate::parser::sema::decl::record::insert_enum_decl;
use crate::types::lex::token_kind::{Keyword, TokenKind};
use crate::types::parser::ast::{
    common::StructOrUnion, DeclKey,
    TypeKey,
};
use crate::types::parser::common::{Ident, IdentList};
use crate::types::parser::decl_spec::{
    DeclSpec, EnumSpec, Enumerator, FuncSpec, ParamDecl, ParamList,
    StorageSpec, StructDeclarator, TypeQual, TypeQuals, TypeSpec,
    TypeSpecKind,
};
use crate::types::parser::declarator::{Declarator, DeclaratorChunk, DeclaratorChunkKind, InitDeclarator};

impl Parser<'_> {
    fn expect_semi_or_lbrace_error(&mut self) -> ParserError {
        let kind = parser_error::ErrorKind::Expect {
            expect: "identifier or '{'".to_owned(),
        };
        self.error_here( kind)
    }

    /// 检查 declarator `{` '(` `[` `ident`
    fn check_declarator(&mut self) -> bool {
        use TokenKind::*;
        let kind = &self.stream.peek().kind;
        match kind {
            LParen | LBrace | LBracket | Ident(_) => true,
            _ => false,
        }
    }

    // 检查指针
    fn check_pointer(&mut self) -> bool {
        let kind = &self.stream.peek().kind;
        matches!(kind, TokenKind::Star)
    }


    /// 解析前缀 declaration 和 function definiton 的共同前缀
    pub(crate) fn parse_decl_prefix(&mut self) -> ParserResult<DeclPrefix> {
        let lo = self.stream.span();
        let decl_spec = self.parse_decl_spec()?;

        let declarator = if self.check( TokenKind::Semi) {
            // 遇到 ; 结束了
            None
        } else {
            let mut declarator = Declarator::new(Rc::clone(&decl_spec));
            self.parse_declarator( &mut declarator)?;
            Some(declarator)
        };

        Ok(DeclPrefix { decl_spec, declarator, lo })
    }

    // 解 declaration
    pub(crate) fn parse_decl(&mut self) -> ParserResult<DeclGroup> {
        let prefix = self.parse_decl_prefix()?;
        self.parse_decl_after_declarator( prefix)
    }

    /// 在已经解析 decl_spec [declarator] 后继续解析 decl
    pub(crate) fn parse_decl_after_declarator(
        &mut self,
        prefix: DeclPrefix,
    ) -> ParserResult<DeclGroup> {
        let mut group = DeclGroup::default();

        if let Some(x) = prefix.declarator {
            // 解析到 declarator 继续解析
            self.parse_init_declarator_list( x, &mut group)?;
        } else {
            // 没有 declarator 结束
            // act on decl_spec
            todo!()
        }

        let _ = self.expect( TokenKind::Semi)?;

        let hi = self.stream.prev_span();
        let span = Span::span(prefix.lo, hi);
        group.span = span;

        Ok(group)
    }

    /// decl spec
    pub(crate) fn parse_decl_spec(&mut self) -> ParserResult<Rc<DeclSpec>> {
        let lo = self.stream.span();

        let mut storages: Vec<StorageSpec> = Vec::new();
        let mut type_quals: Vec<TypeQual> = Vec::new();
        let mut func_specs: Vec<FuncSpec> = Vec::new();
        let mut type_specs: Vec<TypeSpec> = Vec::new();

        loop {
            let token = self.stream.peek();
            if Self::is_storage_spec(token) {
                let spec = StorageSpec::new(self.stream.next());
                storages.push(spec);
                // typedef extern static auto register
            } else if self.is_type_spec( token) {
                // 解析组合下一个 type spec
                let spec = self.parse_type_spec()?;
                type_specs.push(spec);
            } else if Self::is_type_qual(token) {
                // const restrict volatile
                let spec = TypeQual::new(self.stream.next());
                type_quals.push(spec);
            } else if self.check_keyword( Keyword::Inline) {
                // inline
                let spec = self.parse_function_spec()?;
                func_specs.push(spec);
            } else {
                break;
            };
        }
        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        // 肯定不能为空
        debug_assert!(!type_specs.is_empty());

        // 构建 decl_spec
        let builder = DeclSpecBuilder {
            storages,
            type_quals,
            func_specs,
            type_specs,
            span,
        };

        let decl_spec = builder.build(self.ctx)?;

        Ok(decl_spec)
    }

    /// 解析type spec
    fn parse_type_spec(&mut self) -> ParserResult<TypeSpec> {
        let token = self.stream.peek();
        let lo = token.span;

        let kind = match &token.kind {
            // 一定是 typedef (is_type_spec 已经检测过了)
            TokenKind::Ident(_) => {
                // 消耗 token
                let token = self.stream.next();
                let symbol = token.kind.into_ident().unwrap();
                let ident = Ident {
                    symbol,
                    span: token.span,
                };
                let scope = self.ctx
                    .scope_mgr
                    .must_lookup_ident(ident.clone())?;
                let decl_key = scope.get_decl();
                TypeSpecKind::TypeName(ident, decl_key)
            }

            // keyword struct union enum
            TokenKind::Keyword(kw) => match kw {
                Keyword::Struct | Keyword::Union => {
                    // 由这个函数自己消耗 token
                    let spec = self.parse_record_spec()?;
                    TypeSpecKind::Record(spec)
                }
                Keyword::Enum => {
                    // 由这个函数自己消耗 enum token
                    let spec = self.parse_enum_spec()?;
                    TypeSpecKind::Enum(spec)
                }

                // 一定是那堆 keyword
                _ => TypeSpecKind::new(*kw),
            },
            _ => unreachable!(),
        };

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let spec = TypeSpec { kind, span };

        // 组合
        Ok(spec)
    }

    // /// 解析 type qualifier
    // /// - `ctx`: Context
    // /// - `type_qual`: result qualifier. yes, it's output parameter
    // fn parse_type_qual(&mut self) -> ParserResult<()> {
    //     let token = self.stream.next();

    //     let kw = token
    //         .kind
    //         .as_keyword()
    //         .expect("wrong! token is not keyword");

    //     let qual = Some(TypeQual::new(token));

    //     // 追踪原来的 type_qual
    //     let origin = match kw {
    //         Keyword::Const => &mut type_quals.is_const,
    //         Keyword::Restrict => &mut type_quals.is_restrict,
    //         Keyword::Volatile => &mut type_quals.is_volatile,
    //         _ => unreachable!("wrong! token is keyword but not one of const, restrict, volatile"),
    //     };

    //     // 出现重复了，发个Warning
    //     if let Some(x) = origin.as_ref() {
    //         let error = ParserError::duplicate(kw.to_string(), DECL_SPEC, token.span);
    //         ctx.send_error(error)?;
    //     }

    //     Ok(())
    // }

    fn parse_function_spec(&mut self) -> ParserResult<FuncSpec> {
        let inline = self.stream.next();
        let func_spec = FuncSpec::new(inline);
        Ok(func_spec)
    }

    /// 兼容 abstract_declarator
    /// 假设 `int **( (*a)() )[]` 结果应该是 `setname(a) [ * () [] * * ] int`
    /// 解析的时候应该反过来
    pub(crate) fn parse_declarator(&mut self, declarator: &mut Declarator) -> ParserResult<()> {
        let lo = self.stream.span();

        let mut pointers: Vec<DeclaratorChunk> = Vec::new();

        // 解析 pointer 部分
        if self.check_pointer() {
            self.parse_pointer( &mut pointers)?;
        }

        // 解析 direct declarator 部分
        if self.check_declarator() {
            self.parse_direct_declarator( declarator)?;
        }

        // 合并 direct declarator 和 pointer
        // 反转插入
        pointers.reverse();
        declarator.chunks.append(&mut pointers);

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);
        declarator.span = span;

        Ok(())
    }

    /// 解析 direct declarator 的第一步，非循环部分，包括 `ident | (ident)`
    fn parse_direct_declarator_suffix(
        &mut self,
        declarator: &mut Declarator,
    ) -> ParserResult<()> {
        // todo 这里可能有问题， abstract declarator 可能出问题
        if let Some(ident) = self.consume_ident() {
            // 设置name
            let ident = Ident::new(ident);
            declarator.name = Some(ident);
        } else if let Some(_) = self.consume( TokenKind::LParen) {
            // 解析 括号 (xxx)
            self.parse_declarator( declarator)?;
            let _ = self.expect( TokenKind::RParen)?;
        } else {
            unreachable!(
                "parse_direct_declarator_suffix: unexpected {:?}",
                self.stream.peek()
            );
        };

        Ok(())
    }

    /// 解析direct declarator
    fn parse_direct_declarator(&mut self, declarator: &mut Declarator) -> ParserResult<()> {
        // 非循环部分
        self.parse_direct_declarator_suffix( declarator)?;

        // 循环部分 [] ()
        loop {
            let lo = self.stream.span();

            let kind = if let Some(_) = self.consume( TokenKind::LBracket) {
                // array []
                // let type_qual = self.parse_type_qual_list_opt()?;
                // 是否是空括号[]
                let expr = match self.check( TokenKind::RBracket) {
                    true => None,                           // 空括号
                    false => Some(self.parse_assign_expr()?), // 非空解析为表达式
                };
                let _ = self.expect( TokenKind::RBracket)?;
                DeclaratorChunkKind::Array { expr }
            } else if let Some(_lparen) = self.consume( TokenKind::LParen) {
                // func ()

                // 参数类型
                let param = if self.is_type_spec( self.stream.peek()) {
                    // 普通函数参数
                    let list = self.parse_parameter_list()?;
                    ParamDecl::Params(list)
                } else if self.check_ident() {
                    // K&R函数定义参数
                    let idents = self.parse_ident_list()?;
                    ParamDecl::Idents(idents)
                } else {
                    // 没有参数使用默认
                    ParamDecl::Params(ParamList::default())
                };

                let _r = self.expect( TokenKind::RParen)?;

                DeclaratorChunkKind::Function { param }
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

    /// 解析 pointer *
    fn parse_pointer(&mut self, chunks: &mut Vec<DeclaratorChunk>) -> ParserResult<()> {
        loop {
            let lo = self.stream.span();

            if self.consume( TokenKind::Star).is_none() {
                break;
            }

            let type_qual = match Self::is_type_qual(self.stream.peek()) {
                true => self.parse_type_qual_list()?,
                false => TypeQuals::default(),
            };

            let hi = self.stream.prev_span();
            let span = Span::span(lo, hi);

            let kind = DeclaratorChunkKind::Pointer {
                type_quals: type_qual,
            };
            let chunk = DeclaratorChunk::new(kind, span);

            chunks.push(chunk);
        }

        Ok(())
    }

    fn parse_type_qual_list_opt(&mut self) -> ParserResult<Option<TypeQuals>> {
        if Self::is_type_qual(self.stream.peek()) {
            self.parse_type_qual_list().map(|list| Some(list))
        } else {
            Ok(None)
        }
    }

    fn parse_type_qual_list(&mut self) -> ParserResult<TypeQuals> {

        let mut type_quals = TypeQuals::default();
        loop {
            if Self::is_type_qual(self.stream.peek()) {
                let qual = TypeQual::new(self.stream.next());

                // 设置 const restrict volatile
                let field = match &qual.kind {
                    TypeQualKind::Const => &mut type_quals.is_const,
                    TypeQualKind::Restrict => &mut type_quals.is_restrict,
                    TypeQualKind::Volatile => &mut type_quals.is_volatile,
                };

                // 重复发一个警告
                if field.is_some() {
                    let error = ParserError::duplicate(qual.to_string(), DECL_SPEC, qual.span);
                    self.ctx.send_error(error)?;
                }
            } else {
                break;
            }
        }
        Ok(type_quals)
    }

    fn parse_init_declarator_list(
        &mut self,
        declarator: Declarator,
        group: &mut DeclGroup,
    ) -> ParserResult<()> {
        let decl_spec = Rc::clone(&declarator.decl_spec);

        let init = self.parse_init_declarator( Rc::clone(&decl_spec), Some(declarator))?;
        group.decls.push(init);

        while let Some(comma) = self.consume( TokenKind::Comma) {
            let init = self.parse_init_declarator( Rc::clone(&decl_spec), None)?;
            group.decls.push(init);
        }
        Ok(())
    }

    ///
    /// # Arguments
    /// - `decl_spec`: DeclSpec引用
    /// - `declarator`: 传入None表示无Declarator
    fn parse_init_declarator(
        &mut self,
        decl_spec: Rc<DeclSpec>,
        declarator: Option<Declarator>,
    ) -> ParserResult<DeclKey> {
        let lo = self.stream.span();

        // 解析declarator
        let declarator = match declarator {
            Some(x) => x,
            None => {
                let mut declarator = Declarator::new(decl_spec);
                self.parse_declarator( &mut declarator)?;
                declarator
            }
        };

        // 解析initializer部分
        let init = match self.consume( TokenKind::Assign) {
            Some(_) => Some(self.parse_initializer()?),
            None => None,
        };

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let init_declarator = InitDeclarator {
            declarator,
            init,
            span,
        };

        let mut sema = self.sema();
        // 类型检查
        let decl = sema.act_on_init_declarator(init_declarator)?;
        Ok(decl)
    }

    /// 解析 initializer
    fn parse_initializer(&mut self) -> ParserResult<Initializer> {
        let init = if let Some(lparen) = self.consume( TokenKind::LParen) {
            let l = lparen.span.to_pos();
            let inits = self.parse_initializer_list()?;
            let r = self.expect( TokenKind::RParen)?.span.to_pos();
            Initializer::InitList { inits }
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

        while let Some(comma) = self.consume( TokenKind::Comma) {
            if self.check( TokenKind::RParen) {
                break;
            }
            let init = self.parse_initializer()?;
            list.inits.push(init);
        }
        Ok(list)
    }

    /// 解析 record `struct/union [ident]` 部分
    fn parse_record_suffix(&mut self) -> ParserResult<RecordSuffix> {
        let lo = self.stream.span();

        // 消耗struct union关键字
        let kw = self.expect_keyword_pair( Keyword::Struct, Keyword::Union)?;
        let record_kind = StructOrUnion::new(kw);

        let name = self.consume_ident().map(Ident::new); // 尝试解析名字

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        Ok(RecordSuffix {
            record: record_kind,
            name,
            span,
        })
    }

    /// 解析 record
    fn parse_record_spec(&mut self) -> ParserResult<DeclKey> {
        // 解析前缀
        let suffix = self.parse_record_suffix()?;

        // 前向声明，如果没有名字则不做前向声明
        let fwd_decl = match suffix.name {
            Some(x) => Some(insert_record_decl(
                self.ctx,
                suffix.record.clone(),
                x,
                suffix.span,
            )?),
            None => None,
        };

        // 尝试解析定义
        let mut def_decl: Option<DeclKey> = None;
        if self.consume( TokenKind::LBrace).is_some() {
            let group = self.parse_struct_decl_list()?;
            let _ = self.expect( TokenKind::RBrace)?;

            let hi = self.stream.prev_span();
            let span = Span::span(suffix.span, hi);

            def_decl = todo!("act_on_record_def")
        };

        // fwd, def 必须存在一个，否则出错
        let decl = def_decl.or(fwd_decl);
        let decl = match decl {
            Some(x) => x,
            None => {
                let err = ParserError::expect(EXPECT_IDENT_OR_LB, suffix.span);
                return Err(err);
            }
        };

        Ok(decl)
    }

    /// 结构体内部声明，不负责括号，要不要 members 作用域待定
    fn parse_struct_decl_list(&mut self) -> ParserResult<Vec<DeclGroup>> {
        let mut decls = Vec::new();

        if self.check( TokenKind::RBrace) {
            return Ok(decls);
        }

        loop {
            let group = self.parse_struct_decl()?;
            decls.push(group);
            if self.check( TokenKind::RBrace) {
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
        self.parse_struct_declarator_list( &mut group, decl_spec)?;
        let semi = self.expect( TokenKind::Semi)?.span.to_pos();

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);
        group.span = span;

        Ok(group)
    }

    /// 结构体声明declarator列表形如 *a, **b, **c
    fn parse_struct_declarator_list(
        &mut self,
        group: &mut DeclGroup,
        decl_spec: Rc<DeclSpec>,
    ) -> ParserResult<()> {
        // 构建declarator
        let decl = self.parse_struct_declarator( Rc::clone(&decl_spec))?;
        group.decls.push(decl);

        while let Some(_) = self.consume( TokenKind::Comma) {
            let decl = self.parse_struct_declarator( Rc::clone(&decl_spec))?;
            group.decls.push(decl);
        }

        Ok(())
    }

    /// 解析struct的成员，负责插入符号表，应该插入member
    fn parse_struct_declarator(&mut self, decl_spec: Rc<DeclSpec>) -> ParserResult<DeclKey> {
        let mut declarator = Declarator::new(decl_spec);

        let lo = self.stream.span();

        let mut colon = None;
        let mut bit_field = None;

        if self.check_declarator() {
            self.parse_declarator( &mut declarator)?;
        }

        if let Some(colon_token) = self.consume( TokenKind::Colon) {
            colon = Some(colon_token.span.to_pos());
            bit_field = Some(self.parse_assign_expr()?);
        }

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let struct_declarator = StructDeclarator {
            declarator,
            bit_field,
            span,
        };

        let sema = self.sema();
        // 语义分析，获取类型
        let decl = sema.act_on_record_field(struct_declarator)?;
        Ok(decl)
    }

    fn parse_enum_suffix(&mut self) -> ParserResult<EnumSuffix> {
        let lo = self.stream.span();
        self.expect_keyword( Keyword::Enum)?;

        // 检查是否合法
        if self.check_ident() || self.check(TokenKind::LBrace) {
            let kind = parser_error::ErrorKind::Expect {
                expect: "identifier or '{'".to_owned(),
            };
            return Err(self.error_here( kind));
        }

        let name = self.consume_ident().map(Ident::new);

        // 计算一下当前的span，添加 Ref 声明
        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        Ok(EnumSuffix { name, span })
    }

    // todo 重构成 parse_record_spec 的样子
    /// 解析enum声明或定义
    fn parse_enum_spec(&mut self) -> ParserResult<DeclKey> {
        // 准备枚举上下文
        // sema.enter_decl(DeclContextKind::Enum);

        let suffix = self.parse_enum_suffix()?;

        // 前向声明，虽然 enum 定义不需要前向声明
        let fwd_decl = match suffix.name.clone() {
            Some(x) => Some(insert_enum_decl(self.ctx, x, suffix.span)?),
            None => None,
        };

        let def_decl: Option<DeclKey> = None;

        if self.consume( TokenKind::RBrace).is_some() {
            // 解析枚举列表
            let enums = self.parse_enumerator_list()?;
            self.expect( TokenKind::RBrace)?;

            let hi = self.stream.prev_span();
            let span = Span::span(suffix.span, hi);
            let spec = EnumSpec {
                name: suffix.name,
                enums,
                span,
            };

            todo!("act on enum spec def")
        }

        // 二者选其一，必须有 decl, def 优先
        let decl = def_decl.or(fwd_decl);
        let decl = match decl {
            Some(x) => x,
            None => {
                let err = ParserError::expect(EXPECT_IDENT_OR_LB, suffix.span);
                return Err(err);
            }
        };

        Ok(decl)
    }

    fn parse_enumerator_list(&mut self) -> ParserResult<Vec<Enumerator>> {
        let mut enums: Vec<Enumerator> = Vec::new();
        loop {
            let enumerator = self.parse_enumerator()?;
            enums.push(enumerator);

            if self.consume( TokenKind::Comma).is_none() {
                break;
            }
        }
        Ok(enums)
    }

    // 解析枚举的成员，应该是要管理符号表的
    fn parse_enumerator(&mut self) -> ParserResult<Enumerator> {
        let lo = self.stream.span();

        let ident = self.expect_ident()?;
        let name = Ident::new(ident);
        let mut expr = None;
        if let Some(_assign_token) = self.consume( TokenKind::Assign) {
            expr = Some(self.parse_assign_expr()?);
        };

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let enumerator = Enumerator { name, expr, span };
        Ok(enumerator)
    }

    /// 解析函数列表
    /// - 不包含左右括号
    /// - 不负责构建符号表
    /// - 不支持尾部逗号
    fn parse_parameter_list(&mut self) -> ParserResult<ParamList> {
        let lo = self.stream.span();

        let mut params: Vec<DeclKey> = Vec::new();
        let mut is_variadic = false;

        // 解析列表参数声明
        loop {
            let decl = self.parse_parameter_decl()?;
            params.push(decl);
            if self.consume( TokenKind::Comma).is_none() {
                break;
            }

            if self.consume( TokenKind::Ellipsis).is_some() {
                is_variadic = true;
                break;
            }
        }

        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        let list = ParamList {
            params,
            is_variadic,
            span,
        };
        Ok(list)
    }

    /// 解析函数参数声明，不负责插入符号表
    fn parse_parameter_decl(&mut self) -> ParserResult<DeclKey> {
        let lo = self.stream.span();

        // 准备 declarator 结构
        let decl_spec = self.parse_decl_spec()?;
        let mut declarator = Declarator::new(decl_spec);

        // 解析 declarator
        self.parse_declarator( &mut declarator)?;

        // 计算span
        let hi = self.stream.prev_span();
        let span = Span::span(lo, hi);

        declarator.span = span;

        let mut sema = self.sema();
        // 这个函数要进行必要的检测，不负责管理符号表
        let decl = sema.act_on_param_var(declarator)?;

        Ok(decl)
    }

    fn parse_ident_list(&mut self) -> ParserResult<IdentList> {
        let mut list = IdentList::new();
        let ident = self.expect_ident()?;
        let ident = Ident::new(ident);
        list.idents.push(ident);

        while let Some(_) = self.consume( TokenKind::Comma) {
            let ident = self.expect_ident()?;
            let ident = Ident::new(ident);
            list.idents.push(ident);
        }

        Ok(list)
    }

    /// 解析 type name
    pub(crate) fn parse_type_name(&mut self) -> ParserResult<TypeKey> {
        let decl_specs = self.parse_decl_spec()?;
        let mut declarator = Declarator::new(decl_specs);
        if self.check_declarator() {
            self.parse_declarator( &mut declarator)?;
        };

        let mut sema = self.sema();
        let info = sema.resolve_declarator(declarator)?;
        Ok(info.ty)
    }
}