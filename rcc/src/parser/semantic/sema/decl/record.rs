use crate::{err::parser_error::ParserError, parser::{ast::{DeclKey, decl::{Decl, DeclKind}}, common::Ident, comp_ctx::CompCtx, semantic::sema::type_ctx::type_builder::{TypeBuilder, TypeBuilderKind}}, types::span::Span};

pub fn insert_enum_ref(ctx: &mut CompCtx, name: Option<Ident>, span: Span) -> ParserError<DeclKey> {
    let ty = match name.as_ref() {
        None => {
            let kind = TypeBuilderKind::new_enum(ctx);
            let builder = TypeBuilder::new(kind);
            ctx.type_ctx
                .build_type(builder)
                .map_err(|err| ParserError::from_type_error(err, span))?
        }
        Some(x) => match ctx.scope_mgr.lookup_local_tag(x.symbol) {
            None => {
                let kind = TypeBuilderKind::new_enum(ctx);
                let ty = ctx.type_ctx.build_type(kind)?;
                ty
            }
            Some(x) => return x,
        },
    };
    let kind = DeclKind::EnumRef; 
    let decl = Decl{
        storage: None,
        kind,
        name,
        ty,
        span,
    };
     

    ctx.scope_mgr.insert_ident()?
}