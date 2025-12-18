use crate::err::parser_error;
use crate::err::parser_error::{ParserError, ParserResult};
use crate::lex::types::token_kind::{Keyword, TokenKind};
use crate::parser::ast::decl::DeclKey;
use crate::parser::ast::types::TypeKey;
use crate::parser::comp_ctx::CompCtx;
use crate::parser::parser_core::*;
use crate::parser::parser_expr::parse_assign_expr;
use crate::parser::semantic::ast::decl::{DeclGroup, Initializer, InitializerList, StructOrUnion};
use crate::parser::semantic::common::{Ident, IdentList};
use crate::parser::semantic::decl_spec::*;
use crate::parser::semantic::declarator::*;
use crate::parser::semantic::sema::decl::decl_context::DeclContextKind;
use crate::parser::semantic::sema::decl::declarator::PartialDecl;
use crate::types::span::{Pos, Span};
use std::rc::Rc;

macro_rules! dup_error {
    ($ele:expr, $context:expr) => {{
        let item = $ele.kind_str().to_owned();
        let kind = parser_error::ErrorKind::Duplicate {
            item,
            context: $context.to_owned(),
        };
        ParserError::new(kind, $ele.span)
    }};
}

macro_rules! combine_error {
    ($ele:expr, $context:expr) => {{
        let prev = $ele.kind_str().to_owned();
        let kind = parser_error::ErrorKind::NonCombinable {
            prev,
            context: $context.to_owned(),
        };
        ParserError::new(kind, $ele.span)
    }};
}

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
    const CONTEXT: &str = "declaration specifier";

    let lo = ctx.stream.span();

    let mut storage: Option<StorageSpec> = None;
    let mut type_specs: Vec<TypeSpec> = Vec::new();
    let mut type_quals: [Option<TypeQual>; 3] = [None; 3];
    let mut func_spec: Option<FuncSpec> = None;

    loop {
        let token = ctx.stream.peek();
        if is_storage_spec(token) {
            // typedef extern static auto register
            let spec = parse_storage_spec(ctx)?;

            if let Some(storage) = &storage {
                let error = if storage.kind == spec.kind {
                    dup_error!(spec, CONTEXT)
                } else {
                    combine_error!(storage, CONTEXT)
                };
                send_error(ctx, error)
            }
            storage = Some(spec);
        } else if is_type_spec(ctx, token) {
            // 类型
            let spec = parse_type_spec(ctx)?;
            type_specs.push(spec);
        } else if is_type_qual(token) {
            // const restrict volatile
            let qual = parse_type_qual(ctx)?;
            let idx = qual.kind as usize;

            if type_quals[idx].is_some() {
                let error = dup_error!(qual, CONTEXT);
                send_error(ctx, error)
            }

            type_quals[idx] = Some(qual);
        } else if check_keyword(ctx, Keyword::Inline) {
            // inline
            let spec = parse_function_spec(ctx)?;

            if func_spec.is_some() {
                let error = dup_error!(spec, CONTEXT);
                send_error(ctx, error)
            }
            func_spec = Some(spec);
        } else {
            break;
        };
    }

    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);

    let decl_spec = Rc::new(DeclSpec {
        storage,
        type_specs,
        type_quals,
        func_spec,
        span,
    });
    Ok(decl_spec)
}

fn parse_storage_spec(ctx: &mut CompCtx) -> ParserResult<StorageSpec> {
    let token = ctx.stream.next();
    let storage_spec = StorageSpec::new(token);
    Ok(storage_spec)
}

fn parse_type_spec(ctx: &mut CompCtx) -> ParserResult<TypeSpec> {
    let token = ctx.stream.peek();
    let span = token.span;
    let spec = match &token.kind {
        TokenKind::Ident(_) => {
            let symbol = ctx.stream.next().kind.into_ident().unwrap();
            let ident = Ident { symbol, span };
            let decl = match sema.lookup_chain(symbol) {
                Some(x) => x,
                None => todo!(), // 不是类型，出错
            };
            let kind = TypeSpecKind::TypeName(ident, decl);
            TypeSpec { kind, span }
        }
        TokenKind::Keyword(kw) => match kw {
            Keyword::Struct => {
                let spec = parse_struct_or_union_spec(ctx)?;
                let kind = TypeSpecKind::Struct(spec);
                TypeSpec { kind, span }
            }
            Keyword::Union => {
                let spec = parse_struct_or_union_spec(ctx)?;
                let kind = TypeSpecKind::Union(spec);
                TypeSpec { kind, span }
            }
            Keyword::Enum => {
                let spec = parse_enum_spec(ctx)?;
                let kind = TypeSpecKind::Enum(spec);
                TypeSpec { kind, span }
            }
            _ => TypeSpec::new(ctx.stream.next()),
        },
        _ => unreachable!(),
    };
    Ok(spec)
}

fn parse_type_qual(ctx: &mut CompCtx) -> ParserResult<TypeQual> {
    let token = ctx.stream.next();
    let type_qual = TypeQual::new(token);
    Ok(type_qual)
}

fn parse_function_spec(ctx: &mut CompCtx) -> ParserResult<FuncSpec> {
    let inline = ctx.stream.next();
    let func_spec = FuncSpec::new(inline);
    Ok(func_spec)
}

/// 兼容abstract_declarator
pub(crate) fn parse_declarator(ctx: &mut CompCtx, declarator: &mut Declarator) -> ParserResult<()> {
    let lo = ctx.stream.span();

    if check_pointer(ctx) {
        parse_pointer(ctx, declarator)?;
    }
    if check_declarator(ctx) {
        parse_direct_declarator(ctx, declarator)?;
    }

    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);
    declarator.span = span;
    Ok(())
}

fn parse_direct_declarator_first(
    ctx: &mut CompCtx,
    declarator: &mut Declarator,
) -> ParserResult<()> {
    let lo = ctx.stream.span();

    let kind = if let Some(ident) = consume_ident(ctx) {
        let ident = Ident::new(ident);
        declarator.name = Some(ident);
        return Ok(());
    } else if let Some(lparen) = consume(ctx, TokenKind::LParen) {
        let l = lparen.span.to_pos();
        parse_declarator(ctx, declarator)?;
        let r = expect(ctx, TokenKind::RParen)?.span.to_pos();

        DeclaratorChunkKind::Paren { l, r }
    } else {
        println!("{:?}", ctx.stream.next());
        println!("{:?}", ctx.stream.next());
        println!("{:?}", ctx.stream.next());
        unreachable!()
    };

    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);

    let chunk = DeclaratorChunk::new(kind, span);
    declarator.chunks.push(chunk);

    Ok(())
}

fn parse_direct_declarator(ctx: &mut CompCtx, declarator: &mut Declarator) -> ParserResult<()> {
    parse_direct_declarator_first(ctx, declarator)?;

    loop {
        let lo = ctx.stream.span();

        let kind = if let Some(lbracket) = consume(ctx, TokenKind::LBracket) {
            // array []
            let l = lbracket.span.to_pos();
            let type_qual = parse_type_qual_list_opt(ctx)?;
            // 是否是空括号[]
            let expr = match check(ctx, TokenKind::RBracket) {
                true => None,                           // 空括号
                false => Some(parse_assign_expr(ctx)?), // 非空解析为表达式
            };
            let r = expect(ctx, TokenKind::RBracket)?.span.to_pos();
            DeclaratorChunkKind::Array {
                l,
                type_qual,
                expr,
                r,
            }
        } else if let Some(lparen) = consume(ctx, TokenKind::LParen) {
            // func ()
            let l = lparen.span.to_pos();

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

            let r = expect(ctx, TokenKind::RParen)?.span.to_pos();

            DeclaratorChunkKind::Function { l, param, r }
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

fn parse_pointer(ctx: &mut CompCtx, declarator: &mut Declarator) -> ParserResult<()> {
    loop {
        let lo = ctx.stream.span();

        let star = match consume(ctx, TokenKind::Star) {
            Some(x) => x.span.to_pos(),
            None => break,
        };
        let type_qual = match is_type_qual(ctx.stream.peek()) {
            true => parse_type_qual_list(ctx)?,
            false => [None; 3],
        };

        let hi = ctx.stream.prev_span();
        let span = Span::span(lo, hi);

        let kind = DeclaratorChunkKind::Pointer { star, type_qual };
        let chunk = DeclaratorChunk::new(kind, span);

        declarator.chunks.push(chunk);
    }

    Ok(())
}

fn parse_type_qual_list_opt(ctx: &mut CompCtx) -> ParserResult<Option<TypeQualType>> {
    if is_type_qual(ctx.stream.peek()) {
        parse_type_qual_list(ctx).map(|list| Some(list))
    } else {
        Ok(None)
    }
}

fn parse_type_qual_list(ctx: &mut CompCtx) -> ParserResult<TypeQualType> {
    let mut type_qual: [Option<TypeQual>; 3] = [None; 3];
    loop {
        if is_type_qual(ctx.stream.peek()) {
            let qual = TypeQual::new(ctx.stream.next());
            let idx = qual.kind as usize;

            if type_qual[idx].is_some() {
                let error = dup_error!(qual, "Declaration Specifier");
                send_error(ctx, error);
            }

            type_qual[idx] = Some(qual);
        } else {
            break;
        }
    }
    Ok(type_qual)
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
        colon,
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

    // todo 添加声明，目前也许不需要

    let body;
    if let Some(lbrace) = consume(ctx, TokenKind::LBrace) {
        let mut decls = Vec::new();
        let mut commas = Vec::new();

        let l = lbrace.span.to_pos();
        // 解析枚举列表
        parse_enumerator_list(ctx, &mut decls, &mut commas)?;
        let r = expect(ctx, TokenKind::RBrace)?.span.to_pos();
        body = Some(EnumSpecBody {
            l,
            decls,
            commas,
            r,
        });
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
    let decl = sema.act_on_finish_enum(spec)?;
    Ok(decl)
}

fn parse_enumerator_list(
    ctx: &mut CompCtx,
    decls: &mut Vec<DeclKey>,
    commas: &mut Vec<Pos>,
) -> ParserResult<()> {
    loop {
        let decl = parse_enumerator(ctx)?;
        decls.push(decl);

        if let Some(comma) = consume(ctx, TokenKind::Comma) {
            commas.push(comma.span.to_pos());
        } else {
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
    let mut eq = None;
    let mut expr = None;
    if let Some(assign_token) = consume(ctx, TokenKind::Assign) {
        eq = Some(assign_token.span.to_pos());
        expr = Some(parse_assign_expr(ctx)?);
    };

    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);

    let enumerator = Enumerator {
        name,
        eq,
        expr,
        span,
    };
    let decl = sema.act_on_enumerator(enumerator)?;
    Ok(decl)
}

/// 函数列表，不包含左右括号，负责构建符号表
fn parse_parameter_list(ctx: &mut CompCtx) -> ParserResult<ParamList> {
    let lo = ctx.stream.span();

    let mut params: Vec<DeclKey> = Vec::new();
    let mut commas = Vec::new();
    let mut ellipsis = None;

    // 解析第一个参数声明
    let decl = parse_parameter_decl(ctx)?;

    params.push(decl);

    // 解析后续参数声明
    while let Some(comma) = consume(ctx, TokenKind::Comma) {
        commas.push(comma.span.to_pos());
        if let Some(token) = consume(ctx, TokenKind::Ellipsis) {
            ellipsis = Some(token.span);
            break;
        }
        let decl = parse_parameter_decl(ctx)?;
        params.push(decl);
    }

    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);

    let list = ParamList {
        params,
        commas,
        ellipsis,
        span,
    };
    Ok(list)
}

/// 解析函数参数声明，负责插入符号表
fn parse_parameter_decl(ctx: &mut CompCtx) -> ParserResult<DeclKey> {
    let lo = ctx.stream.span();

    let decl_spec = parse_decl_spec(ctx)?;
    let mut declarator = Declarator::new(decl_spec);
    parse_declarator(ctx, &mut declarator)?;

    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);

    declarator.span = span;

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
