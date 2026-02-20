use crate::err::scope_error::ScopeSource;
use crate::parser::ast::decls::decl::{DeclGroup, RecordDecl};
use crate::parser::comp_ctx::CompCtx;
use crate::parser::sema::scope::lookup::{conflict_error_if, lookup_or_insert_decl};
use crate::parser::sema::type_ctx::type_ctx::TypeCtx;
use crate::parser::sema::Sema;
use crate::types::parser::ast::common::{RecordKind, StructOrUnion};
use crate::types::parser::ast::decls::decl::{Decl, DeclKind};
use crate::types::parser::ast::types::type_builder::TypeBuilder;
use crate::types::parser::ast::{DeclKey, TypeKey};
use crate::types::parser::common::Ident;
use crate::types::parser::decl_spec::StorageSpec;
use crate::{
    err::parser_error::{ParserError, ParserResult},
    types::span::Span,
};

/// Record 对象构建辅助结构
/// - `kind`: struct or union
/// - `name`: 结构体名称
/// - `fields`: 成员
/// - `ty`: 类型，如果没有为 None
/// - `storage`: storage spec
/// - `span`: 位置
pub struct RecordHelper {
    pub kind: StructOrUnion,
    pub name: Option<Ident>,
    pub fields: Option<Vec<DeclGroup>>,
    pub ty: Option<TypeKey>,
    pub storage: Option<StorageSpec>,
    pub span: Span,
}

impl Sema {
    /// decl 是否是 enum ，如果不是返回 DeclNotMatch 错误
    fn is_enum(type_ctx: &TypeCtx, ty: TypeKey) -> bool {
        let ty = type_ctx.get_type(ty);
        ty.kind.is_enum()
    }

    /// decl 是否是 record ，如果不是返回 DeclNotMatch 错误
    fn is_record(type_ctx: &TypeCtx, kind: RecordKind, ty: TypeKey) -> bool {
        let ty = type_ctx.get_type(ty);
        ty.kind
            .as_record()
            .map(|(kind1, _, _)| *kind1 == kind)
            .unwrap_or(false)
    }

    /// 在当前作用域插入 enum 声明
    pub fn insert_enum_decl(ctx: &mut CompCtx, name: Ident, span: Span) -> ParserResult<DeclKey> {
        // 查询同级是否已经存在声明
        let symbol = ctx.scope_mgr.lookup_local_tag(name.symbol);
        let ty = match symbol {
            Some(x) => {
                // 检查是否为同一个
                conflict_error_if(
                    Sema::is_enum(&ctx.type_ctx, x.ty),
                    &name,
                    x.get_decl(),
                    ScopeSource::Tag,
                )?;
                x.ty
            }
            None => {
                // 不存在，构建类型
                let kind = ctx.type_ctx.new_enum_builder();
                let builder = TypeBuilder::new(kind);
                ctx.type_ctx
                    .build_type(builder)
                    .map_err(|err| ParserError::from_type_error(err, span))?
            }
        };

        // 构造 DeclDecl
        let kind = DeclKind::EnumDef { enums: None };
        let decl = Decl {
            storage: None,
            kind,
            name: Some(name.clone()),
            ty,
            span,
        };

        // 存入池子
        let decl_key = Sema::insert_decl(decl);

        // 插入符号表
        let enum_def = lookup_or_insert_decl(ctx, decl_key, ty, ScopeSource::Tag);

        // 设置 definition
        let decl = ctx.get_decl_mut(decl_key);
        match &mut decl.kind {
            DeclKind::EnumDecl { def } => *def = enum_def,
            _ => unreachable!(),
        }

        Ok(decl_key)
    }

    /// 构建 record 类型，Type 不存在则分配
    pub fn build_record(ctx: &mut CompCtx, record: RecordHelper) -> DeclKey {
        let name = record.name;

        let record_kind = record.kind;
        let record_decl = RecordDecl::new(record_kind, record.fields);
        let decl_kind = DeclKind::Record(record_decl);
        let ty = record
            .ty
            .unwrap_or_else(|| ctx.type_ctx.new_record(record_kind.kind));

        let storage = record.storage;
        let span = record.span;

        let decl = Decl::new(name, decl_kind, ty, storage, span);
        ctx.insert_decl(decl)
    }

    // /// 在当前作用域插入 record 声明
    // pub fn insert_record_decl(
    //     ctx: &mut CompCtx,
    //     record: StructOrUnion,
    //     name: Ident,
    //     span: Span,
    // ) -> ParserResult<DeclKey> {
    //     // 查询同级是否已经存在声明
    //     let symbol = ctx.scope_mgr.lookup_local_tag(name.symbol);
    //     let ty = match symbol {
    //         Some(x) => {
    //             // 检查tag声明是否相同
    //             conflict_error_if(
    //                 Sema::is_record(&ctx.type_ctx, record.kind, x.ty),
    //                 &name,
    //                 x.get_decl(),
    //                 ScopeSource::Tag,
    //             )?;
    //             x.ty
    //         }
    //         None => {
    //             // 不存在，构建类型
    //             let kind = ctx.type_ctx.new_enum();
    //             let builder = TypeBuilder::new(kind);
    //             ctx.type_ctx
    //                 .build_type(builder)
    //                 .map_err(|err| ParserError::from_type_error(err, span))?
    //         }
    //     };
    //
    //     // 构造 DeclDecl
    //     let kind = DeclKind::RecordDecl {
    //         kind: record,
    //         def: None,
    //     };
    //     let decl = Decl {
    //         storage: None,
    //         kind,
    //         name: Some(name.clone()),
    //         ty,
    //         span,
    //     };
    //
    //     // 存入池子
    //     let decl_key = ctx.insert_decl(decl);
    //
    //     // 插入符号表
    //     let record_def = lookup_or_insert_decl(ctx, decl_key, ty, ScopeSource::Tag);
    //
    //     // 设置 definition
    //     let decl = ctx.get_decl_mut(decl_key);
    //     match &mut decl.kind {
    //         DeclKind::EnumDecl { def } => *def = record_def,
    //         _ => unreachable!(),
    //     }
    //
    //     Ok(decl_key)
    // }
    //
    // /// 填充 record 的前向声明
    // pub fn fill_record_fwd_ref(ctx: &mut CompCtx, definition: DeclKey, decls: Vec<DeclKey>) {
    //     for decl in decls.into_iter() {
    //         let decl = ctx.get_decl_mut(decl);
    //         debug_assert!(decl.kind.is_record_decl());
    //         match &mut decl.kind {
    //             DeclKind::RecordDecl { def, .. } => {
    //                 *def = Some(definition);
    //             }
    //             _ => unreachable!(),
    //         }
    //     }
    // }
    //
    // /// 插入 record 定义
    // pub fn insert_record_def(
    //     ctx: &mut CompCtx,
    //     kind: DeclKind,
    //     name: Ident,
    //     span: Span,
    // ) -> ParserResult<DeclKey> {
    //     debug_assert!(kind.is_record_def());
    //     let record_kind = match &kind {
    //         DeclKind::RecordDef { kind, .. } => kind.kind.clone(),
    //         _ => unreachable!(),
    //     };
    //
    //     let symbol = ctx.scope_mgr.lookup_local_tag(name.symbol);
    //     let ty = match symbol {
    //         Some(x) => {
    //             // 检查tag声明是否相同
    //             conflict_error_if(
    //                 is_record(ctx, record_kind, x.ty),
    //                 &name,
    //                 x.get_decl(),
    //                 ScopeSource::Tag,
    //             )?;
    //             x.ty
    //         }
    //         None => {
    //             // 不存在，构建类型
    //             let kind = ctx.type_ctx.new_enum();
    //             let builder = TypeBuilder::new(kind);
    //             ctx.type_ctx
    //                 .build_type(builder)
    //                 .map_err(|err| ParserError::from_type_error(err, span))?
    //         }
    //     };
    //
    //     // 构建 decl
    //     let decl = Decl {
    //         storage: None,
    //         kind,
    //         name: Some(name.clone()),
    //         ty,
    //         span,
    //     };
    //
    //     let def = ctx.insert_decl(decl);
    //
    //     // 添加到符号表
    //     let decls = lookup_or_insert_def(ctx, def, ty, ScopeSource::Tag)?;
    //     // 填充前向引用
    //     fill_record_fwd_ref(ctx, def, decls);
    //
    //     Ok(def)
    // }
}
