use crate::err::parser_error;
use crate::constant::str::DECL_SPEC;
use crate::err::parser_error::{ParserError, ParserResult};
use crate::lex::types::token_kind::{Keyword, TokenKind};
use crate::parser::ast::{DeclKey, TypeKey};
use crate::parser::comp_ctx::CompCtx;
use crate::parser::parser_core::*;
use crate::parser::parser_expr::parse_assign_expr;
use crate::parser::semantic::ast::decl::{DeclGroup, Initializer, InitializerList, StructOrUnion};
use crate::parser::semantic::common::{Ident, IdentList};
use crate::parser::semantic::decl_spec::*;
use crate::parser::semantic::declarator::*;
use crate::parser::semantic::sema::decl::decl_context::DeclContextKind;
use crate::parser::semantic::sema::decl::declarator::{PartialDecl, act_on_decl_spec};
use crate::parser::semantic::sema::type_ctx::type_builder::{TypeBuilder, TypeBuilderKind};
use crate::types::span::{Pos, Span};
use std::rc::Rc;


fn check_declarator(ctx: &CompCtx) -> bool {
    let kind = &ctx.stream.peek().kind;
    match kind {
        TokenKind::LParen | TokenKind::LBrace | TokenKind::LBracket | TokenKind::Ident(_) => true,
        _ => false,
    }
}

fn check_pointer(ctx: &CompCtx) -> bool {
    let kind = &ctx.stream.peek().kind;
    matches!(kind, TokenKind::Star)
}

pub(crate) fn parse_decl(ctx: &mut CompCtx) -> ParserResult<DeclGroup> {
    let lo = ctx.stream.span();

    let decl_spec = parse_decl_spec(ctx)?;
    let mut declarator = Declarator::new(decl_spec);
    parse_declarator(ctx, &mut declarator)?;

    parse_decl_after_declarator(ctx, lo, declarator)
}

pub(crate) fn parse_decl_after_declarator(
    ctx: &mut CompCtx,
    lo: Span,
    declarator: Declarator,
) -> ParserResult<DeclGroup> {
    let mut group = DeclGroup::default();
    parse_init_declarator_list(ctx, declarator, &mut group)?;
    let semi = expect(ctx, TokenKind::Semi)?.span.to_pos();

    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);
    group.semi = semi;
    group.span = span;

    Ok(group)
}

pub(crate) fn parse_decl_spec(ctx: &mut CompCtx) -> ParserResult<Rc<DeclSpec>> {
    let lo = ctx.stream.span();

    let mut storages: Vec<StorageSpec> = Vec::new();
    let mut type_quals: Vec<TypeQual> = Vec::new();
    let mut func_specs: Vec<FuncSpec> = Vec::new();
    let mut specs: Vec<TypeSpec> = Vec::new();

    loop {
        let token = ctx.stream.peek();
        if is_storage_spec(token) {
            let spec = StorageSpec::new(ctx.stream.next());
            storages.push(spec);
            // typedef extern static auto register
        } else if is_type_spec(ctx, token) {
            // 解析组合下一个 type spec
            let spec = parse_type_spec(ctx)?;
            specs.push(spec);
        } else if is_type_qual(token) {
            // const restrict volatile
            let spec = TypeQual::new(ctx.stream.next());
            type_quals.push(spec);
        } else if check_keyword(ctx, Keyword::Inline) {
            // inline
            let spec = parse_function_spec(ctx)?;
            func_specs.push(spec);
        } else {
            break;
        };
    }

    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);

    // 构建 decl_spec
    let decl_spec = act_on_decl_spec(
        ctx, 
        storages, 
        type_quals, 
        func_specs, 
        type_specs, 
        span
    )?;

    Ok(decl_spec)
}

/// 解析type spec
fn parse_type_spec(
    ctx: &mut CompCtx,
) -> ParserResult<TypeSpec> {
    let token = ctx.stream.peek();
    let span = token.span;
    let spec = match &token.kind {
        // 一定是 typedef (is_type_spec 已经检测过了)
        TokenKind::Ident(_) => {
            // 消耗 token
            let token = ctx.stream.next();
            let symbol = token.kind.into_ident().unwrap();
            let ident = Ident { symbol, span };
            let decl = ctx
                .scope_mgr
                .must_lookup_ident(symbol)
                .map_err(|x| ParserError::from_scope_error(x, span))?;
            let kind = TypeSpecKind::TypeName(ident, decl);
            TypeSpec { kind, span }
        }

        // keyword struct union enum
        TokenKind::Keyword(kw) => match kw {
            Keyword::Struct => {
                // 由这个函数自己消耗 struct token
                let spec = parse_struct_or_union_spec(ctx)?;
                let kind = TypeSpecKind::Record(spec);
                TypeSpec { kind, span }
            }
            Keyword::Union => {
                // 由这个函数自己消耗 union token
                let spec = parse_struct_or_union_spec(ctx)?;
                let kind = TypeSpecKind::Record(spec);
                TypeSpec { kind, span }
            }
            Keyword::Enum => {
                // 由这个函数自己消耗 enum token
                let spec = parse_enum_spec(ctx)?;
                let kind = TypeSpecKind::Enum(spec);
                TypeSpec { kind, span }
            }

            // 一定是那堆 keyword
            _ => TypeSpec::new(ctx.stream.next()),
        },
        _ => unreachable!(),
    };

    // 组合
    Ok(spec)
}

/// 解析 type qualifier
/// - `ctx`: Context
/// - `type_qual`: result qualifier. yes, it's output parameter
fn parse_type_qual(ctx: &mut CompCtx) -> ParserResult<()> {
    let token = ctx.stream.next();

    let kw = token
        .kind
        .as_keyword()
        .expect("wrong! token is not keyword");
    
    let qual = Some(TypeQual::new(token));

    // 追踪原来的 type_qual
    let origin = match kw {
        Keyword::Const => {
            &mut type_quals.is_const
        }
        Keyword::Restrict => {
            &mut type_quals.is_restrict
        }
        Keyword::Volatile => {
            &mut type_quals.is_volatile
        }
        _ => unreachable!("wrong! token is keyword but not one of const, restrict, volatile"),
    };

    // 出现重复了，发个Warning
    if let Some(x) = origin.as_ref() {
        let error =ParserError::duplicate(
            kw.to_string(), 
            DECL_SPEC,
            token.span
        );
        ctx.send_error(error)?;
    }

    Ok(())
}

fn parse_function_spec(ctx: &mut CompCtx) -> ParserResult<FuncSpec> {
    let inline = ctx.stream.next();
    let func_spec = FuncSpec::new(inline);
    Ok(func_spec)
}

/// 兼容abstract_declarator
pub(crate) fn parse_declarator(ctx: &mut CompCtx, declarator: &mut Declarator) -> ParserResult<()> {
    let lo = ctx.stream.span();

    let mut pointers: Vec<DeclaratorChunk> = Vec::new();

    // 解析 pointer
    if check_pointer(ctx) {
        parse_pointer(ctx, &mut pointers)?;
    }

    // 解析 direct declarator
    if check_declarator(ctx) {
        parse_direct_declarator(ctx, declarator)?;
    }

    // 反转插入
    pointers.reverse();
    declarator.chunks.append(&mut pointers);

    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);
    declarator.span = span;

    Ok(())
}

/// 解析 direct declarator 的第一步，非循环部分
fn parse_direct_declarator_first(
    ctx: &mut CompCtx,
    declarator: &mut Declarator,
) -> ParserResult<()> {
    if let Some(ident) = consume_ident(ctx) {
        // 设置name
        let ident = Ident::new(ident);
        declarator.name = Some(ident);
    } else if let Some(_) = consume(ctx, TokenKind::LParen) {
        // 解析 括号 (xxx)
        parse_declarator(ctx, declarator)?;
        let _ = expect(ctx, TokenKind::RParen)?;
    } else {
        unreachable!(
            "parse_direct_declarator_first: unexpected {:?}",
            ctx.stream.peek()
        );
    };

    Ok(())
}

/// 解析direct declarator
fn parse_direct_declarator(ctx: &mut CompCtx, declarator: &mut Declarator) -> ParserResult<()> {
    // 非循环部分
    parse_direct_declarator_first(ctx, declarator)?;

    // 循环部分 [] ()
    loop {
        let lo = ctx.stream.span();

        let kind = if let Some(_lbracket) = consume(ctx, TokenKind::LBracket) {
            // array []
            // let type_qual = parse_type_qual_list_opt(ctx)?;
            // 是否是空括号[]
            let expr = match check(ctx, TokenKind::RBracket) {
                true => None,                           // 空括号
                false => Some(parse_assign_expr(ctx)?), // 非空解析为表达式
            };
            let _rbracket = expect(ctx, TokenKind::RBracket)?;
            DeclaratorChunkKind::Array { expr }
        } else if let Some(_lparen) = consume(ctx, TokenKind::LParen) {
            // func ()

            // 参数类型
            let param = if is_type_spec(ctx, ctx.stream.peek()) {
                // 普通函数参数
                let list = parse_parameter_list(ctx)?;
                ParamDecl::Params(list)
            } else if check_ident(ctx) {
                // K&R函数定义参数
                let idents = parse_ident_list(ctx)?;
                ParamDecl::Idents(idents)
            } else {
                // 没有参数使用默认
                ParamDecl::Params(ParamList::default())
            };

            let _r = expect(ctx, TokenKind::RParen)?;

            DeclaratorChunkKind::Function { param }
        } else {
            break;
        };

        let hi = ctx.stream.prev_span();
        let span = Span::span(lo, hi);

        let chunk = DeclaratorChunk::new(kind, span);
        declarator.chunks.push(chunk)
    }

    Ok(())
}

/// 解析 pointer *
fn parse_pointer(ctx: &mut CompCtx, chunks: &mut Vec<DeclaratorChunk>) -> ParserResult<()> {
    loop {
        let lo = ctx.stream.span();

        if consume(ctx, TokenKind::Star).is_none() {
            break;
        }

        let type_qual = match is_type_qual(ctx.stream.peek()) {
            true => parse_type_qual_list(ctx)?,
            false => TypeQuals::default(),
        };

        let hi = ctx.stream.prev_span();
        let span = Span::span(lo, hi);

        let kind = DeclaratorChunkKind::Pointer {
            type_quals: type_qual,
        };
        let chunk = DeclaratorChunk::new(kind, span);

        chunks.push(chunk);
    }

    Ok(())
}

fn parse_type_qual_list_opt(ctx: &mut CompCtx) -> ParserResult<Option<TypeQuals>> {
    if is_type_qual(ctx.stream.peek()) {
        parse_type_qual_list(ctx).map(|list| Some(list))
    } else {
        Ok(None)
    }
}

fn parse_type_qual_list(ctx: &mut CompCtx) -> ParserResult<TypeQuals> {
    use TypeQualKind::*;

    let mut type_quals = TypeQuals::default();
    loop {
        if is_type_qual(ctx.stream.peek()) {
            let qual = TypeQual::new(ctx.stream.next());

            // 设置 const restrict volatile
            let field = match &qual.kind {
                Const => {
                    &mut type_quals.is_const
                }
                Restrict => {
                    &mut type_quals.is_restrict 
                }
                Volatile => {
                    &mut type_quals.is_volatile
                }
            };

            // 重复发一个警告
            if field.is_some() {
                let error = ParserError::duplicate(
                    qual.to_string(),
                    DECL_SPEC,
                    qual.span,
                );
                send_error(ctx, error);
            }
        } else {
            break;
        }
    }
    Ok(type_quals)
}

fn parse_init_declarator_list(
    ctx: &mut CompCtx,
    declarator: Declarator,
    group: &mut DeclGroup,
) -> ParserResult<()> {
    let decl_spec = Rc::clone(&declarator.decl_spec);

    let init = parse_init_declarator(ctx, Rc::clone(&decl_spec), Some(declarator))?;
    group.decls.push(init);

    while let Some(comma) = consume(ctx, TokenKind::Comma) {
        let init = parse_init_declarator(ctx, Rc::clone(&decl_spec), None)?;
        group.commas.push(comma.span.to_pos());
        group.decls.push(init);
    }
    Ok(())
}

///
/// # Arguments
/// - `decl_spec`: DeclSpec引用
/// - `declarator`: 传入None表示无Declarator
fn parse_init_declarator(
    ctx: &mut CompCtx,
    decl_spec: Rc<DeclSpec>,
    declarator: Option<Declarator>,
) -> ParserResult<DeclKey> {
    let lo = ctx.stream.span();

    // 解析declarator
    let declarator = match declarator {
        Some(x) => x,
        None => {
            let mut declarator = Declarator::new(decl_spec);
            parse_declarator(ctx, &mut declarator)?;
            declarator
        }
    };
    let mut eq: Option<Pos> = None;
    let mut init: Option<Initializer> = None;
    if let Some(assign_token) = consume(ctx, TokenKind::Assign) {
        // 解析initializer部分
        eq = Some(assign_token.span.to_pos());
        init = Some(parse_initializer(ctx)?);
    }

    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);

    let init_declarator = InitDeclarator {
        declarator,
        eq,
        init,
        span,
    };
    let decl = sema.act_on_init_declarator(init_declarator)?;
    Ok(decl)
}

fn parse_initializer(ctx: &mut CompCtx) -> ParserResult<Initializer> {
    let init = if let Some(lparen) = consume(ctx, TokenKind::LParen) {
        let l = lparen.span.to_pos();
        let inits = parse_initializer_list(ctx)?;
        let r = expect(ctx, TokenKind::RParen)?.span.to_pos();
        Initializer::InitList { l, inits, r }
    } else {
        let expr = parse_assign_expr(ctx)?;
        Initializer::Expr(expr)
    };
    Ok(init)
}

fn parse_initializer_list(ctx: &mut CompCtx) -> ParserResult<InitializerList> {
    let mut list = InitializerList::new();
    let init = parse_initializer(ctx)?;
    list.inits.push(init);

    while let Some(comma) = consume(ctx, TokenKind::Comma) {
        if check(ctx, TokenKind::RParen) {
            break;
        }
        let init = parse_initializer(ctx)?;
        list.commas.push(comma.span.to_pos());
        list.inits.push(init);
    }
    Ok(list)
}

fn parse_struct_or_union_spec(ctx: &mut CompCtx) -> ParserResult<DeclKey> {
    let lo = ctx.stream.span();

    // 消耗struct union关键字
    let kw = expect_keyword_pair(ctx, Keyword::Struct, Keyword::Union)?;
    let record_kind = StructOrUnion::new(kw);
    let name = consume_ident(ctx).map(Ident::new); // 尝试解析名字
    let mut body = None;

    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);

    // 进入struct上下文
    sema.enter_decl(DeclContextKind::Record);

    // 如果没有名字不需要处理声明
    if let Some(name) = &name {
        sema.act_on_record_ref(record_kind.clone(), name.clone(), span)?;
    }

    // 尝试解析内部声明
    if let Some(lbrace) = consume(ctx, TokenKind::LBrace) {
        let r = lbrace.span.to_pos();
        let group = parse_struct_decl_list(ctx)?;
        let l = expect(ctx, TokenKind::RBrace)?.span.to_pos();
        body = Some(StructSpecBody {
            r,
            groups: group,
            l,
        })
    }

    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);

    // 语义分析
    let spec = StructSpec {
        kind: record_kind,
        name,
        body,
        span,
    };

    let decl = sema.act_on_finish_record(spec)?;
    Ok(decl)
}

/// 结构体内部声明，不负责括号
fn parse_struct_decl_list(ctx: &mut CompCtx) -> ParserResult<Vec<DeclGroup>> {
    let mut decls = Vec::new();

    if check(ctx, TokenKind::RBrace) {
        return Ok(decls);
    }

    loop {
        let group = parse_struct_decl(ctx)?;
        decls.push(group);
        if check(ctx, TokenKind::RBrace) {
            break;
        }
    }

    Ok(decls)
}

/// 结构体成员声明，包括结尾分号
fn parse_struct_decl(ctx: &mut CompCtx) -> ParserResult<DeclGroup> {
    let lo = ctx.stream.span();

    let decl_spec = parse_decl_spec(ctx)?;
    let mut group = DeclGroup::default();
    parse_struct_declarator_list(ctx, &mut group, decl_spec)?;
    let semi = expect(ctx, TokenKind::Semi)?.span.to_pos();

    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);
    group.semi = semi;
    group.span = span;

    Ok(group)
}

/// 结构体声明declarator列表形如 *a, **b, **c
fn parse_struct_declarator_list(
    ctx: &mut CompCtx,
    group: &mut DeclGroup,
    decl_spec: Rc<DeclSpec>,
) -> ParserResult<()> {
    // 构建declarator
    let decl = parse_struct_declarator(ctx, Rc::clone(&decl_spec))?;
    group.decls.push(decl);

    while let Some(comma) = consume(ctx, TokenKind::Comma) {
        let comma = comma.span.to_pos();
        let decl = parse_struct_declarator(ctx, Rc::clone(&decl_spec))?;
        group.decls.push(decl);
        group.commas.push(comma);
    }

    Ok(())
}

/// 解析struct的成员，负责插入符号表
fn parse_struct_declarator(ctx: &mut CompCtx, decl_spec: Rc<DeclSpec>) -> ParserResult<DeclKey> {
    let mut declarator = Declarator::new(decl_spec);

    let lo = ctx.stream.span();

    let mut colon = None;
    let mut bit_field = None;

    if check_declarator(ctx) {
        parse_declarator(ctx, &mut declarator)?;
    }

    if let Some(colon_token) = consume(ctx, TokenKind::Colon) {
        colon = Some(colon_token.span.to_pos());
        bit_field = Some(parse_assign_expr(ctx)?);
    }

    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);

    let struct_declarator = StructDeclarator {
        declarator,
        bit_field,
        span,
    };

    // 语义分析，获取类型
    let decl = sema.act_on_record_field(struct_declarator)?;
    Ok(decl)
}

/// 解析enum声明或定义
fn parse_enum_spec(ctx: &mut CompCtx) -> ParserResult<DeclKey> {
    // 准备枚举上下文
    sema.enter_decl(DeclContextKind::Enum);
    let lo = ctx.stream.span();

    let kw = expect_keyword(ctx, Keyword::Enum)?.span;

    // 检查是否合法
    if check_ident(ctx) || check(ctx, TokenKind::LBrace) {
        let kind = parser_error::ErrorKind::Expect {
            expect: "identifier or '{'".to_owned(),
        };
        return Err(error_here(ctx, kind));
    }

    let name = consume_ident(ctx).map(Ident::new);

   

    // todo 如果没有 name 向类型和符号添加一次前向声明，如果有 name 就查询符号表

    let body;
    if let Some(lbrace) = consume(ctx, TokenKind::LBrace) {
        let mut decls = Vec::new();
        // let mut commas = Vec::new();

        let _l = lbrace.span.to_pos();
        // 解析枚举列表
        parse_enumerator_list(ctx, &mut decls)?;
        let _r = expect(ctx, TokenKind::RBrace)?.span.to_pos();
        body = Some(decls);
    } else {
        // 出错
        let expect = "identifier or '{'".to_owned();
        let kind = parser_error::ErrorKind::Expect { expect };
        let error = error_here(ctx, kind);
        return Err(error);
    }

    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);

    let spec = EnumSpec {
        enum_span: kw,
        name,
        body,
        span,
    };
    // 完成并结束枚举上下文
    Ok(decl)
}

fn parse_enumerator_list(ctx: &mut CompCtx, decls: &mut Vec<DeclKey>) -> ParserResult<()> {
    loop {
        let decl = parse_enumerator(ctx)?;
        decls.push(decl);

        if consume(ctx, TokenKind::Comma).is_none() {
            break;
        }
    }
    Ok(())
}

// 解析枚举的成员，负责插入符号表
fn parse_enumerator(ctx: &mut CompCtx) -> ParserResult<DeclKey> {
    let lo = ctx.stream.span();

    let ident = expect_ident(ctx)?;
    let name = Ident::new(ident);
    let mut expr = None;
    if let Some(_assign_token) = consume(ctx, TokenKind::Assign) {
        expr = Some(parse_assign_expr(ctx)?);
    };

    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);

    let enumerator = Enumerator { name, expr, span };
    let decl = sema.act_on_enumerator(enumerator)?;
    Ok(decl)
}

/// 函数列表，不包含左右括号，负责构建符号表
fn parse_parameter_list(ctx: &mut CompCtx) -> ParserResult<ParamList> {
    let lo = ctx.stream.span();

    let mut params: Vec<DeclKey> = Vec::new();
    let mut is_variadic = false;

    // 解析第一个参数声明
    let decl = parse_parameter_decl(ctx)?;

    params.push(decl);

    // 解析后续参数声明
    while let Some(comma) = consume(ctx, TokenKind::Comma) {
        if let Some(token) = consume(ctx, TokenKind::Ellipsis) {
            is_variadic = true;
            break;
        }
        let decl = parse_parameter_decl(ctx)?;
        params.push(decl);
    }

    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);

    let list = ParamList {
        params,
        is_variadic,
        span,
    };
    Ok(list)
}

/// 解析函数参数声明，负责插入符号表
fn parse_parameter_decl(ctx: &mut CompCtx) -> ParserResult<DeclKey> {
    let lo = ctx.stream.span();

    // 准备 declarator 结构
    let decl_spec = parse_decl_spec(ctx)?;
    let mut declarator = Declarator::new(decl_spec);

    // 解析 declarator
    parse_declarator(ctx, &mut declarator)?;

    // 计算span
    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);

    declarator.span = span;

    // 这个函数要进行必要的检测，但一定不负责管理符号表
    let decl = sema.act_on_param_var(declarator)?;

    Ok(decl)
}

fn parse_ident_list(ctx: &mut CompCtx) -> ParserResult<IdentList> {
    let mut list = IdentList::new();
    let ident = expect_ident(ctx)?;
    let ident = Ident::new(ident);
    list.idents.push(ident);

    while let Some(comma) = consume(ctx, TokenKind::Comma) {
        let ident = expect_ident(ctx)?;
        let ident = Ident::new(ident);
        list.idents.push(ident);
        list.commas.push(comma.span.to_pos());
    }

    Ok(list)
}

pub(crate) fn parse_type_name(ctx: &mut CompCtx) -> ParserResult<TypeKey> {
    let lo = ctx.stream.span();

    let decl_specs = parse_decl_spec(ctx)?;
    let mut declarator = Declarator::new(decl_specs);
    if check_declarator(ctx) {
        parse_declarator(ctx, &mut declarator)?;
    };

    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);

    declarator.span = span;

    let PartialDecl { ty_key: ty, .. } = sema.act_on_declarator(declarator)?;
    Ok(ty)
}
