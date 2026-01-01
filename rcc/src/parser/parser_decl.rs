use std::rc::Rc;

use crate::constant::str::EXPECT_IDENT_OR_LB;
use crate::parser::parser_core::error_here;
use crate::parser::semantic::decl_spec::{EnumSuffix, TypeQualKind};
use crate::parser::semantic::declarator::DeclPrefix;
use crate::parser::semantic::sema::decl::declarator::act_on_init_declarator;
use crate::{
    constant::str::DECL_SPEC,
    err::parser_error::{self, ParserError, ParserResult},
    lex::types::token_kind::{Keyword, TokenKind},
    parser::{
        ast::{
            DeclKey, TypeKey,
            common::StructOrUnion,
            decl::{DeclGroup, Initializer, InitializerList},
        },
        common::{Ident, IdentList},
        comp_ctx::CompCtx,
        parser_core::{
            check, check_ident, check_keyword, consume, consume_ident, expect, expect_ident,
            expect_keyword, expect_keyword_pair, is_storage_spec, is_type_qual, is_type_spec,
        },
        parser_expr::parse_assign_expr,
        semantic::{
            decl_spec::{
                DeclSpec, EnumSpec, Enumerator, FuncSpec, ParamDecl, ParamList,
                StorageSpec, StructDeclarator, TypeQual, TypeQuals, TypeSpec,
                TypeSpecKind,
            },
            declarator::{Declarator, DeclaratorChunk, DeclaratorChunkKind, InitDeclarator},
            sema::decl::{
                declarator::{DeclSpecBuilder, PartialDecl},
                record::{insert_enum_decl, insert_record_decl},
            },
        },
    },
    types::span::Span,
};

fn expect_semi_or_lbrace_error(ctx: &CompCtx) -> ParserError {
    let kind = parser_error::ErrorKind::Expect {
        expect: "identifier or '{'".to_owned(),
    };
    error_here(ctx, kind)
}

/// 检查 declarator `{` '(` `[` `ident`
fn check_declarator(ctx: &CompCtx) -> bool {
    use TokenKind::*;
    let kind = &ctx.stream.peek().kind;
    match kind {
        LParen | LBrace | LBracket | Ident(_) => true,
        _ => false,
    }
}

// 检查指针
fn check_pointer(ctx: &CompCtx) -> bool {
    let kind = &ctx.stream.peek().kind;
    matches!(kind, TokenKind::Star)
}


/// 解析前缀 declaration 和 function definiton 的共同前缀
pub(crate) fn parse_decl_prefix(ctx: &mut CompCtx) -> ParserResult<DeclPrefix>{
    let lo = ctx.stream.span();
    let decl_spec = parse_decl_spec(ctx)?;

    let declarator = if check(ctx, TokenKind::Semi) {
        // 遇到 ; 结束了
        None 
    } else {
        let mut declarator = Declarator::new(Rc::clone(&decl_spec))
        parse_declarator(ctx, &mut declarator)?;
        Some(declarator)
    };

    Ok(DeclPrefix { decl_spec, declarator, lo })
}

// 解 declaration
pub(crate) fn parse_decl(ctx: &mut CompCtx) -> ParserResult<DeclGroup> {
    let prefix = parse_decl_prefix(ctx)?;
    parse_decl_after_declarator(ctx, prefix)
}

/// 在已经解析 decl_spec [declarator] 后继续解析 decl
pub(crate) fn parse_decl_after_declarator(
    ctx: &mut CompCtx,
    prefix: DeclPrefix,
) -> ParserResult<DeclGroup> {
    let mut group = DeclGroup::default();

    if let Some(x) = prefix.declarator {
        // 解析到 declarator 继续解析
        parse_init_declarator_list(ctx, x, &mut group)?;
    } else {
        // 没有 declarator 结束
        // act on decl_spec
        todo!()
    }

    let _ = expect(ctx, TokenKind::Semi)?;

    let hi = ctx.stream.prev_span();
    let span = Span::span(prefix.lo, hi);
    group.span = span;

    Ok(group)
}

/// decl spec
pub(crate) fn parse_decl_spec(ctx: &mut CompCtx) -> ParserResult<Rc<DeclSpec>> {
    let lo = ctx.stream.span();

    let mut storages: Vec<StorageSpec> = Vec::new();
    let mut type_quals: Vec<TypeQual> = Vec::new();
    let mut func_specs: Vec<FuncSpec> = Vec::new();
    let mut type_specs: Vec<TypeSpec> = Vec::new();

    loop {
        let token = ctx.stream.peek();
        if is_storage_spec(token) {
            let spec = StorageSpec::new(ctx.stream.next());
            storages.push(spec);
            // typedef extern static auto register
        } else if is_type_spec(ctx, token) {
            // 解析组合下一个 type spec
            let spec = parse_type_spec(ctx)?;
            type_specs.push(spec);
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

    // 肯定不能为空
    assert!(!type_specs.is_empty());

    // 构建 decl_spec
    let builder = DeclSpecBuilder {
        storages,
        type_quals,
        func_specs,
        type_specs,
        span,
    };

    let decl_spec = builder.build(ctx)?;

    Ok(decl_spec)
}

/// 解析type spec
fn parse_type_spec(ctx: &mut CompCtx) -> ParserResult<TypeSpec> {
    let token = ctx.stream.peek();
    let lo = token.span;

    let kind = match &token.kind {
        // 一定是 typedef (is_type_spec 已经检测过了)
        TokenKind::Ident(_) => {
            // 消耗 token
            let token = ctx.stream.next();
            let symbol = token.kind.into_ident().unwrap();
            let ident = Ident {
                symbol,
                span: token.span,
            };
            let decl = ctx
                .scope_mgr
                .must_lookup_ident(symbol)
                .map_err(|x| ParserError::from_scope_error(x, token.span))?;
            TypeSpecKind::TypeName(ident, decl)
        }

        // keyword struct union enum
        TokenKind::Keyword(kw) => match kw {
            Keyword::Struct | Keyword::Union => {
                // 由这个函数自己消耗 token
                let spec = parse_record_spec(ctx)?;
                TypeSpecKind::Record(spec)
            }
            Keyword::Enum => {
                // 由这个函数自己消耗 enum token
                let spec = parse_enum_spec(ctx)?;
                TypeSpecKind::Enum(spec)
            }

            // 一定是那堆 keyword
            _ => TypeSpecKind::new(*kw),
        },
        _ => unreachable!(),
    };

    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);

    let spec = TypeSpec { kind, span };

    // 组合
    Ok(spec)
}

// /// 解析 type qualifier
// /// - `ctx`: Context
// /// - `type_qual`: result qualifier. yes, it's output parameter
// fn parse_type_qual(ctx: &mut CompCtx) -> ParserResult<()> {
//     let token = ctx.stream.next();

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

fn parse_function_spec(ctx: &mut CompCtx) -> ParserResult<FuncSpec> {
    let inline = ctx.stream.next();
    let func_spec = FuncSpec::new(inline);
    Ok(func_spec)
}

/// 兼容 abstract_declarator
/// 假设 `int **( (*a)() )[]` 结果应该是 `setname(a) [ * () [] * * ] int`
/// 解析的时候应该反过来
pub(crate) fn parse_declarator(ctx: &mut CompCtx, declarator: &mut Declarator) -> ParserResult<()> {
    let lo = ctx.stream.span();

    let mut pointers: Vec<DeclaratorChunk> = Vec::new();

    // 解析 pointer 部分
    if check_pointer(ctx) {
        parse_pointer(ctx, &mut pointers)?;
    }

    // 解析 direct declarator 部分
    if check_declarator(ctx) {
        parse_direct_declarator(ctx, declarator)?;
    }

    // 合并 direct declarator 和 pointer
    // 反转插入
    pointers.reverse();
    declarator.chunks.append(&mut pointers);

    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);
    declarator.span = span;

    Ok(())
}

/// 解析 direct declarator 的第一步，非循环部分，包括 `ident | (ident)`
fn parse_direct_declarator_suffix(
    ctx: &mut CompCtx,
    declarator: &mut Declarator,
) -> ParserResult<()> {
    // todo 这里可能有问题， abstract declarator 可能出问题
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
            "parse_direct_declarator_suffix: unexpected {:?}",
            ctx.stream.peek()
        );
    };

    Ok(())
}

/// 解析direct declarator
fn parse_direct_declarator(ctx: &mut CompCtx, declarator: &mut Declarator) -> ParserResult<()> {
    // 非循环部分
    parse_direct_declarator_suffix(ctx, declarator)?;

    // 循环部分 [] ()
    loop {
        let lo = ctx.stream.span();

        let kind = if let Some(_) = consume(ctx, TokenKind::LBracket) {
            // array []
            // let type_qual = parse_type_qual_list_opt(ctx)?;
            // 是否是空括号[]
            let expr = match check(ctx, TokenKind::RBracket) {
                true => None,                           // 空括号
                false => Some(parse_assign_expr(ctx)?), // 非空解析为表达式
            };
            let _ = expect(ctx, TokenKind::RBracket)?;
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
                Const => &mut type_quals.is_const,
                Restrict => &mut type_quals.is_restrict,
                Volatile => &mut type_quals.is_volatile,
            };

            // 重复发一个警告
            if field.is_some() {
                let error = ParserError::duplicate(qual.to_string(), DECL_SPEC, qual.span);
                ctx.send_error(error)?;
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

    // 解析initializer部分
    let init = match consume(ctx, TokenKind::Assign) {
        Some(_) => Some(parse_initializer(ctx)?),
        None => None,
    };

    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);

    let init_declarator = InitDeclarator {
        declarator,
        init,
        span,
    };

    // 类型检查
    let decl = act_on_init_declarator(ctx, init_declarator)?;
    Ok(decl)
}

/// 解析 initalizer
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

/// 解析 record `struct/union [ident]` 部分
fn parse_record_suffix(ctx: &mut CompCtx) -> ParserResult<RecordSuffix> {
    let lo = ctx.stream.span();

    // 消耗struct union关键字
    let kw = expect_keyword_pair(ctx, Keyword::Struct, Keyword::Union)?;
    let record_kind = StructOrUnion::new(kw);

    let name = consume_ident(ctx).map(Ident::new); // 尝试解析名字

    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);

    Ok(RecordSuffix {
        record: record_kind,
        name,
        span,
    })
}

/// 解析 record
fn parse_record_spec(ctx: &mut CompCtx) -> ParserResult<DeclKey> {
    // 解析前缀
    let suffix = parse_record_suffix(ctx)?;

    // 前向声明，如果没有名字则不做前向声明
    let fwd_decl = match suffix.name {
        Some(x) => Some(insert_record_decl(
            ctx,
            suffix.record.clone(),
            x,
            suffix.span,
        )?),
        None => None,
    };

    // 尝试解析定义
    let mut def_decl: Option<DeclKey> = None;
    if consume(ctx, TokenKind::LBrace).is_some() {
        let group = parse_struct_decl_list(ctx)?;
        let _ = expect(ctx, TokenKind::RBrace)?;

        let hi = ctx.stream.prev_span();
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

/// 解析struct的成员，负责插入符号表，应该插入member
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

fn parse_enum_suffix(ctx: &mut CompCtx) -> ParserResult<EnumSuffix> {
    let lo = ctx.stream.span();
    expect_keyword(ctx, Keyword::Enum)?;

    // 检查是否合法
    if check_ident(ctx) || check(ctx, TokenKind::LBrace) {
        let kind = parser_error::ErrorKind::Expect {
            expect: "identifier or '{'".to_owned(),
        };
        return Err(error_here(ctx, kind));
    }

    let name = consume_ident(ctx).map(Ident::new);

    // 计算一下当前的span，添加 Ref 声明
    let hi = ctx.stream.prev_span();
    let span = Span::span(lo, hi);

    Ok(EnumSuffix { name, span })
}

// todo 重构成 parse_record_spec 的样子
/// 解析enum声明或定义
fn parse_enum_spec(ctx: &mut CompCtx) -> ParserResult<DeclKey> {
    // 准备枚举上下文
    // sema.enter_decl(DeclContextKind::Enum);

    let suffix = parse_enum_suffix(ctx)?;

    // 前向声明，虽然 enum 定义不需要前向声明
    let fwd_decl = match suffix.name.clone() {
        Some(x) => Some(insert_enum_decl(ctx, x, suffix.span)?),
        None => None,
    };

    let mut def_decl: Option<DeclKey> = None;

    if consume(ctx, TokenKind::RBrace).is_some() {
        // 解析枚举列表
        let enums = parse_enumerator_list(ctx)?;
        expect(ctx, TokenKind::RBrace)?;

        let hi = ctx.stream.prev_span();
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

fn parse_enumerator_list(ctx: &mut CompCtx) -> ParserResult<Vec<Enumerator>> {
    let mut enums: Vec<Enumerator> = Vec::new();
    loop {
        let enumerator = parse_enumerator(ctx)?;
        enums.push(enumerator);

        if consume(ctx, TokenKind::Comma).is_none() {
            break;
        }
    }
    Ok(enums)
}

// 解析枚举的成员，应该是要管理符号表的
fn parse_enumerator(ctx: &mut CompCtx) -> ParserResult<Enumerator> {
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
    Ok(enumerator)
}

/// 解析函数列表
/// - 不包含左右括号
/// - 不负责构建符号表
/// - 不支持尾部逗号
fn parse_parameter_list(ctx: &mut CompCtx) -> ParserResult<ParamList> {
    let lo = ctx.stream.span();

    let mut params: Vec<DeclKey> = Vec::new();
    let mut is_variadic = false;

    // 解析列表参数声明
    loop {
        let decl = parse_parameter_decl(ctx)?;
        params.push(decl);
        if consume(ctx, TokenKind::Comma).is_none() {
            break;
        }

        if consume(ctx, TokenKind::Ellipsis).is_some() {
            is_variadic = true;
            break;
        }
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

/// 解析函数参数声明，不负责插入符号表
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

    // 这个函数要进行必要的检测，不负责管理符号表
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
