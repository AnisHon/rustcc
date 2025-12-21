// use std::cmp;

// use crate::{
//     constant::typ::{DEFAULT_ALIGN, DEFAULT_SIZE},
//     err::parser_error::ParserResult,
//     parser::{
//         ast::{
//             DeclKey, TypeKey,
//             types::{RecordField, Type, TypeKind},
//         },
//         comp_ctx::CompCtx,
//         semantic::decl_spec::{EnumSpec, StructSpec},
//     },
// };

// /// 计算偏移量
// /// # Members
// /// - `offset`: 偏移量
// /// - `align`: 类型的对齐期望
// fn calc_offset(offset: u64, align: u64) -> u64 {
//     // 计算需要的对齐填充
//     let padding = (align - (offset % align)) % align;

//     // 返回对齐后的偏移量
//     offset + padding
// }

// // 解析 record 成员
// pub fn resolve_record_field(
//     ctx: &mut CompCtx,
//     decl: DeclKey,
//     offset: u64,
// ) -> ParserResult<RecordField> {
//     let field = ctx.get_decl(decl);
//     assert!(field.kind.is_record_field());

//     let bit_field = field
//         .kind
//         .as_record_field()
//         .expect("resolve_record_field: not a record")
//         .clone();

//     let name = field.name.clone().map(|x| x.symbol);
//     let ty_key = field.ty;

//     // 如果不是int constant会出错
//     let field = RecordField {
//         name,
//         ty: ty_key,
//         bit_field,
//         offset,
//     };
//     Ok(field)
// }

// /// 将 StructSpec 转换成Type
// pub fn resolve_record_type(ctx: &mut CompCtx, spec: &StructSpec) -> ParserResult<TypeKey> {
//     let mut offset: u64 = 0;
//     let mut fields = Vec::new();

//     let body = match &spec.body {
//         None => todo!(),
//         Some(x) => x,
//     };

//     for group in body.groups.iter() {
//         for decl in group.decls.iter().cloned() {
//             let field = resolve_record_field(ctx, decl, offset)?;

//             // 设置对齐，如果对齐出错，默认为 DEFAULT ALIGN
//             let align = ctx.get_type(field.ty).align(ctx).unwrap_or(DEFAULT_ALIGN);
//             offset = calc_offset(offset, align);
//             fields.push(field);
//         }
//     }

//     // offset 始终是下一个所以可以等价于大小, 最小为DefaulSize
//     let size = cmp::max(DEFAULT_SIZE, offset);

//     let kind = match spec.kind.kind {
//         RecordKind::Struct => TypeKind::Struct {
//             name: spec.name.clone(),
//             fields,
//             size,
//         },
//         RecordKind::Union => TypeKind::Union {
//             name: spec.name.clone(),
//             fields,
//             size,
//         },
//     };

//     // 构建 struct 类型
//     let ty = Type::new(kind);
//     let ty = ctx.type_ctx.from_builder(ty);

//     Ok(ty)
// }

// /// 解析枚举类型
// pub fn resolve_enum(ctx: &mut CompCtx, spec: &EnumSpec) -> ParserResult<TypeKey> {
//     let name = spec.name.clone();

//     // 通过判断是否有 decls(enum body) 确定类型
//     let kind = match spec.decls.is_some() {
//         true => TypeKind::Enum { name },
//         false => {
//             let name = name.expect("impossible anonymous enum declaration");
//             TypeKind::EnumRef { name }
//         }
//     };

//     let ty = Type::new(kind);
//     let ty = ctx.type_ctx.from_builder(ty);
//     Ok(ty)
// }
