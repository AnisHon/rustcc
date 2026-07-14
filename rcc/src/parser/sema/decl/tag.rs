use crate::errors::parser::decl_error::{
    DeclDifferentTypeError, DeclRedefinitionError, DeclResult,
};
use crate::parser::ast::decls::decl::{DeclGroup, DeclStatus, EnumDecl, RecordDecl};
use crate::parser::ast::types::QualType;
use crate::parser::comp_ctx::CompCtx;
use crate::parser::sema::scope;
use crate::types::parser::ast::common::StructOrUnion;
use crate::types::parser::ast::decls::decl::{Decl, DeclKind};
use crate::types::parser::ast::DeclKey;
use crate::types::parser::common::Ident;
use crate::types::parser::decl_spec::Enumerator;
use crate::types::span::Span;

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
    pub span: Span,
}

impl RecordHelper {
    pub fn new_decl(kind: StructOrUnion, name: Option<Ident>, span: Span) -> Self {
        Self {
            kind,
            name,
            fields: None,
            span,
        }
    }

    pub fn new_def(
        kind: StructOrUnion,
        name: Option<Ident>,
        fields: Vec<DeclGroup>,
        span: Span,
    ) -> Self {
        Self {
            kind,
            name,
            fields: Some(fields),
            span,
        }
    }

    pub fn get_status(&self) -> DeclStatus {
        if let Some(_) = self.fields.as_ref() {
            DeclStatus::Definition
        } else {
            DeclStatus::Declaration
        }
    }
}

pub struct EnumHelper {
    pub name: Option<Ident>,
    pub fields: Option<Vec<Enumerator>>,
    pub span: Span,
}

impl EnumHelper {
    pub fn new_decl(name: Option<Ident>, span: Span) -> Self {
        Self {
            name,
            fields: None,
            span,
        }
    }

    pub fn new_def(name: Option<Ident>, fields: Vec<Enumerator>, span: Span) -> Self {
        Self {
            name,
            fields: Some(fields),
            span,
        }
    }

    pub fn get_status(&self) -> DeclStatus {
        if let Some(_) = self.fields.as_ref() {
            DeclStatus::Definition
        } else {
            DeclStatus::Declaration
        }
    }
}

// /// decl 是否是 enum ，如果不是返回 DeclNotMatch 错误
// fn is_enum(type_ctx: &TypeCtx, ty: TypeKey) -> bool {
//     let ty = type_ctx.get_type(ty);
//     ty.kind.is_enum()
// }
//
// /// decl 是否是 record ，如果不是返回 DeclNotMatch 错误
// fn is_record(type_ctx: &TypeCtx, kind: RecordKind, ty: TypeKey) -> bool {
//     let ty = type_ctx.get_type(ty);
//     ty.kind
//         .as_record()
//         .map(|(kind1, _, _)| *kind1 == kind)
//         .unwrap_or(false)
// }
//
// /// 在当前作用域插入 enum 声明
// pub fn insert_enum_decl(ctx: &mut CompCtx, name: Ident, span: Span) -> ParserResult<DeclKey> {
//     // 查询同级是否已经存在声明
//     let symbol = ctx.scope_mgr.lookup_local_tag(name.symbol);
//     let ty = match symbol {
//         Some(x) => {
//             // 检查是否为同一个
//             conflict_error_if(
//                 Sema::is_enum(&ctx.type_ctx, x.ty),
//                 &name,
//                 x.get_decl(),
//                 ScopeSource::Tag,
//             )?;
//             x.ty
//         }
//         None => {
//             // 不存在，构建类型
//             let kind = ctx.type_ctx.new_enum_builder();
//             let builder = TypeBuilder::new(kind);
//             ctx.type_ctx
//                 .build_type(builder)
//                 .map_err(|err| ParserError::from_type_error(err, span))?
//         }
//     };
//
//     // 构造 DeclDecl
//     let kind = DeclKind::EnumDef { enums: None };
//     let decl = Decl {
//         storage: None,
//         kind,
//         name: Some(name.clone()),
//         ty,
//         span,
//     };
//
//     // 存入池子
//     let decl_key = Sema::insert_decl(decl);
//
//     // 插入符号表
//     let enum_def = lookup_or_insert_decl(ctx, decl_key, ty, ScopeSource::Tag);
//
//     // 设置 definition
//     let decl = ctx.get_decl_mut(decl_key);
//     match &mut decl.kind {
//         DeclKind::EnumDecl { def } => *def = enum_def,
//         _ => unreachable!(),
//     }
//
//     Ok(decl_key)
// }

/// 构建 record 类型，Type 不存在则分配
fn build_record(ctx: &mut CompCtx, record: RecordHelper, qual_type: QualType) -> DeclKey {
    let record_kind = record.kind;

    let record_decl = RecordDecl::new(record_kind, record.fields);
    let decl_kind = DeclKind::Record(record_decl);

    let decl = Decl::new(record.name, decl_kind, qual_type, None, record.span);
    ctx.insert_decl(decl)
}

/// 检查record类型匹配和重定义问题
///
/// # Returns
/// - 返回当前定义的 enum 类型
/// - `DifferentType`错误
/// - `Redefinition` 错误
///
fn check_record_symbol(ctx: &mut CompCtx, helper: &RecordHelper) -> DeclResult<QualType> {
    // 查符号表
    let symbol = match helper.name.as_ref() {
        None => None,
        Some(name) => ctx.scope_mgr.lookup_tag(name),
    };

    let symbol = match symbol {
        None => {
            // 符号表无记录，不会出错
            let type_key = ctx.type_ctx.new_enum();
            let qual_type = QualType::from(type_key);
            return Ok(qual_type);
        }
        Some(x) => x,
    };

    // 符号表有记录
    let status = helper.get_status();
    let qual_type = symbol.ty;
    let ty = ctx.type_ctx.get_type(qual_type.ty);
    let name = helper.name.expect("never");
    let is_same_type = ty
        .get_record_kind()
        .map(|x| x == helper.kind.kind)
        .unwrap_or(false);

    // 检查类型
    if !is_same_type {
        // 不是相同类型
        let prev = symbol.get_decl();
        Err(DeclDifferentTypeError::new(status, prev, name).into())
    } else if let Some(def) = symbol.def
        && status == DeclStatus::Definition
    {
        // 重复定义
        Err(DeclRedefinitionError::new(def, name).into())
    } else {
        // 一切正常
        Ok(qual_type)
    }
}

/// 构建 enum Decl 对象，插入符号表
pub fn act_on_record(ctx: &mut CompCtx, helper: RecordHelper) -> DeclResult<DeclKey> {
    // 声明还是定义
    let status = helper.get_status();

    // 检查并获取类型
    let qual_type = check_record_symbol(ctx, &helper)?;

    // 构建 decl 对象
    let decl = build_record(ctx, helper, qual_type);

    // 插入符号表
    match status {
        DeclStatus::Definition => complete_record_decl(ctx, decl, qual_type),
        DeclStatus::Declaration => scope::insert_tag_decl_unchecked(ctx, decl),
    };

    Ok(decl)
}

/// 完成 record 定义
fn complete_record_decl(ctx: &mut CompCtx, decl_key: DeclKey, qual_type: QualType) {
    scope::insert_tag_def_unchecked(ctx, decl_key);
    todo!("填充type, 调用type_ctx::complete_record_type");
}

fn build_enum(ctx: &mut CompCtx, helper: EnumHelper, ty: QualType) -> DeclKey {
    let enum_decl = EnumDecl::new_decl();
    let decl_kind = DeclKind::Enum(enum_decl);

    let decl = Decl::new(helper.name, decl_kind, ty, None, helper.span);
    let decl = ctx.insert_decl(decl);
    decl
}

/// 检查enum类型匹配和重定义问题
///
/// # Returns
/// - 返回当前定义的 enum 类型
/// - `DifferentType`错误
/// - `Redefinition` 错误
///
fn check_enum_symbol(ctx: &mut CompCtx, helper: &EnumHelper) -> DeclResult<QualType> {
    // 查符号表
    let symbol = match helper.name.as_ref() {
        None => None,
        Some(name) => ctx.scope_mgr.lookup_tag(name),
    };

    let symbol = match symbol {
        None => {
            // 符号表无记录，不会出错
            let type_key = ctx.type_ctx.new_enum();
            let qual_type = QualType::from(type_key);
            return Ok(qual_type);
        }
        Some(x) => x,
    };

    // 符号表有记录
    let status = helper.get_status();
    let qual_type = symbol.ty;
    let ty = ctx.type_ctx.get_type(qual_type.ty);
    let name = helper.name.expect("never");

    // 检查类型
    if !ty.is_enum_type() {
        // 不是枚举类型
        let prev = symbol.get_decl();
        Err(DeclDifferentTypeError::new(status, prev, name).into())
    } else if let Some(def) = symbol.def
        && status == DeclStatus::Definition
    {
        // 重复定义
        Err(DeclRedefinitionError::new(def, name).into())
    } else {
        // 一切正常
        Ok(qual_type)
    }
}

/// 构建 enum Decl 对象，插入符号表，填充Type结构
pub fn act_on_enum(ctx: &mut CompCtx, helper: EnumHelper) -> DeclResult<DeclKey> {
    // 声明还是定义
    let status = helper.get_status();

    // 检查并获取类型
    let qual_type = check_enum_symbol(ctx, &helper)?;

    // 构建 decl 对象
    let decl = build_enum(ctx, helper, qual_type);

    // 插入符号表
    match status {
        DeclStatus::Definition => complete_enum_decl(ctx, decl, qual_type),
        DeclStatus::Declaration => scope::insert_tag_decl_unchecked(ctx, decl),
    };

    Ok(decl)
}

/// 完成枚举定义
fn complete_enum_decl(ctx: &mut CompCtx, decl_key: DeclKey, qual_type: QualType) {
    scope::insert_tag_def_unchecked(ctx, decl_key);
    todo!("填充type, 调用type_ctx::complete_enum_type");
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
//                 .map_err(|errors| ParserError::from_type_error(errors, span))?
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
//                 .map_err(|errors| ParserError::from_type_error(errors, span))?
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
