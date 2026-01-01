use crate::constant::str::{DECL_SPEC, TYPEDEF_REQUIRE_NAME};
use crate::err::parser_error::{ParserError, ParserResult};
use crate::parser::ast::decl::{Decl, DeclKind, Initializer};
use crate::parser::ast::types::{FloatSize, IntegerSize, TypeKind};
use crate::parser::ast::{DeclKey, TypeKey};
use crate::parser::common::TypeSpecState;
use crate::parser::comp_ctx::CompCtx;
use crate::parser::semantic::common::Ident;
use crate::parser::semantic::decl_spec::{
    DeclSpec, FuncSpec, StorageSpec, StorageSpecKind, TypeQual, TypeQualKind, TypeQuals, TypeSpec,
    TypeSpecKind,
};
use crate::parser::semantic::declarator::InitDeclarator;
use crate::parser::semantic::sema::scope::scope_struct::{ScopeKind, ScopeSymbol};
use crate::parser::semantic::sema::type_ctx::declarator::{DeclInfo, resolve_declarator};
use crate::parser::semantic::sema::type_ctx::type_builder::TypeBuilderKind;
use crate::types::span::Span;
use std::collections::hash_map::Entry;
use std::rc::Rc;

#[derive(Debug)]
pub struct PartialDecl {
    pub storage: Option<StorageSpec>,
    pub name: Option<Ident>,
    pub ty_key: TypeKey,
}

pub struct DeclSpecBuilder {
    pub storages: Vec<StorageSpec>,
    pub type_quals: Vec<TypeQual>,
    pub func_specs: Vec<FuncSpec>,
    pub type_specs: Vec<TypeSpec>,
    pub span: Span,
}

/// 将 typedef 插入符号表，负责处理名字问题，类型不匹配问题
/// todo: 可能放到 scope 模块更合适
fn insert_typedef(ctx: &mut CompCtx, decl_key: DeclKey) -> ParserResult<()> {
    let decl = ctx.get_decl(decl_key);
    let ty = decl.ty;
    let name = match &decl.name {
        Some(x) => x.clone(),
        None => {
            // typedef 但是没有名字给一个 warning
            let warning = ParserError::warning(TYPEDEF_REQUIRE_NAME.to_owned(), decl.span);
            ctx.send_error(warning)?;
            return Ok(()); // 名字都没有不用了
        }
    };

    match ctx.scope_mgr.entry_local_ident(name.symbol) {
        Entry::Occupied(mut x) => {
            let symbol = x.get_mut();
            // 声明的 type 不同错误
            if symbol.ty != ty {
                let error = ParserError::redefinition(symbol.get_decl(), name);
                return Err(error);
            }
            symbol.def = Some(decl_key); // todo: 目前打算先覆盖
        }
        Entry::Vacant(x) => {
            // 不存在，构造符号表
            let name = *x.key();
            x.insert(ScopeSymbol {
                name,
                decls: Vec::new(),
                def: Some(decl_key),
                ty,
            });
        }
    }

    Ok(())
}

fn default_storage_kind(ctx: &CompCtx) -> StorageSpecKind {
    use ScopeKind::*;
    match ctx.scope_mgr.get_kind() {
        File => StorageSpecKind::Extern,
        Function => StorageSpecKind::Auto,
        Block => StorageSpecKind::Auto,
        ParamList => StorageSpecKind::Auto,
        Record => unreachable!("record should not have storage class"),
    }
}

/// 是否为 typedef 声明
fn is_typedef(storage: Option<&StorageSpec>) -> bool {
    storage
        .as_ref()
        .map(|x| x.kind.is_typedef())
        .unwrap_or(false)
}

// 是否是定义 todo 使用 DefinitionKind 表示 Tentative 定义
fn is_definition(ctx: &mut CompCtx, decl_info: &DeclInfo, has_init: bool) -> bool {
    use ScopeKind::*;
    // 表示显式定义了 extern，不是隐式的
    let extern_kw = decl_info
        .storage
        .as_ref()
        .map(|x| x.kind.is_extern())
        .unwrap_or(false);

    // 判断是否是声明
    match ctx.scope_mgr.get_kind() {
        File => has_init || !extern_kw, // 如果有init一定是定义，如果没有，且没有声明 extern 默认是临时定义
        Function | Block => !extern_kw, // 这种作用域下，只 extern 才是声明，而且 extern 不允许有初始化
        ParamList | Record => unreachable!("param_list and record are not supported"),
    }
}

// 处理 typedef
fn act_on_typedef(ctx: &mut CompCtx, decl_info: DeclInfo, has_init: bool) -> ParserResult<DeclKey> {
    assert!(is_typedef(decl_info.storage.as_ref())); // 必须是 typedef

    // typedef 不能初始化
    if has_init {
        let storage = decl_info.storage.expect("impossible");
        let ident = decl_info.name.expect("with init, but no name?");
        let error = ParserError::illegal_init(storage.to_string(), ident.symbol, storage.span);
        return Err(error);
    }

    // 构造 decl
    let decl = Decl {
        storage: decl_info.storage,
        name: decl_info.name,
        kind: DeclKind::TypeDef,
        ty: decl_info.ty,
        span: decl_info.span,
    };
    let decl_key = ctx.insert_decl(decl);

    // 插入符号表，自动处理名字和类型不匹配问题
    insert_typedef(ctx, decl_key)?;

    Ok(decl_key)
}

pub fn act_on_init_declarator(
    ctx: &mut CompCtx,
    init_declarator: InitDeclarator,
) -> ParserResult<DeclKey> {
    // let declarator = init_declarator.declarator;
    // let name = declarator.name.clone();

    // 构建类型
    let decl_info = resolve_declarator(ctx, init_declarator.declarator)?;


    let has_init = init_declarator.init.is_some();

    // typedef 需要特殊处理
    if is_typedef(decl_info.storage.as_ref()) {
        return act_on_typedef(ctx, decl_info, has_init)
    } 

    // 是否是定义
    let is_def = is_definition(ctx, &decl_info, has_init);

    // 检查 init
    todo!();

    // 构建decl
    todo!();

    // todo: 插入符号表
    todo!();
}

impl DeclSpecBuilder {
    pub fn build(self, ctx: &mut CompCtx) -> ParserResult<Rc<DeclSpec>> {
        let storage = Self::act_on_storages(self.storages)?;
        let type_quals = Self::act_on_type_quals(self.type_quals)?;
        let func_spec = Self::act_on_func_specs(self.func_specs)?;
        let kind = Self::act_on_type_specs(ctx, self.type_specs)?;

        let decl_spec = Rc::new(DeclSpec {
            storage,
            type_quals,
            func_spec,
            kind,
            span: self.span,
        });

        Ok(decl_spec)
    }

    fn act_on_storages(storages: Vec<StorageSpec>) -> ParserResult<Option<StorageSpec>> {
        let mut storage: Option<StorageSpec> = None;
        for spec in storages {
            if let Some(x) = storage {
                let err = ParserError::duplicate(x.to_string(), DECL_SPEC, spec.span);
                return Err(err);
            }
            storage = Some(spec);
        }

        Ok(storage)
    }

    fn act_on_type_quals(quals: Vec<TypeQual>) -> ParserResult<TypeQuals> {
        use TypeQualKind::*;
        let mut res = TypeQuals::default();
        for qual in quals {
            let field = match qual.kind {
                Const => &mut res.is_const,
                Restrict => &mut res.is_restrict,
                Volatile => &mut res.is_volatile,
            };

            if let Some(x) = field {
                let err = ParserError::duplicate(x.to_string(), DECL_SPEC, qual.span);
                return Err(err);
            }
            *field = Some(qual);
        }

        Ok(res)
    }

    fn act_on_func_specs(specs: Vec<FuncSpec>) -> ParserResult<Option<FuncSpec>> {
        let mut func_spec: Option<FuncSpec> = None;
        for spec in specs {
            if let Some(x) = func_spec {
                let err = ParserError::duplicate(x.to_string(), DECL_SPEC, spec.span);
                return Err(err);
            }
            func_spec = Some(spec);
        }

        Ok(func_spec)
    }

    /// 检查type spec是否正确
    fn check_type_spec(
        ctx: &mut CompCtx,
        spec: &TypeSpec,
        int_cnt: i32,
        is_signed: Option<&TypeSpec>,
    ) -> ParserResult<()> {
        use TypeSpecKind::*;
        match &spec.kind {
            Int if int_cnt > 1 => {
                // 多个 int 报错
                let err = ParserError::non_combinable(spec.to_string(), DECL_SPEC, spec.span);
                return Err(err);
            }
            Float | Double | Record(_) | Enum(_) | TypeName(_, _) if is_signed.is_some() => {
                let prev = is_signed.expect("impossible").to_string();
                let err = ParserError::non_combinable(prev, DECL_SPEC, spec.span);
                return Err(err);
            }
            Signed | Unsigned if is_signed.is_some() => {
                let prev = is_signed.expect("impossible").to_string();
                let x = is_signed.unwrap();
                if x.is(&spec.kind) {
                    // 相同，重复
                    let err = ParserError::duplicate(prev, DECL_SPEC, spec.span);
                    ctx.send_error(err)?;
                } else {
                    // 不同，错误
                    let err = ParserError::non_combinable(prev, DECL_SPEC, spec.span);
                    return Err(err);
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn act_on_type_specs(ctx: &mut CompCtx, specs: Vec<TypeSpec>) -> ParserResult<TypeBuilderKind> {
        use TypeSpecKind::*;
        assert!(!specs.is_empty());
        let mut state = TypeSpecState::Init;
        let mut decl: Option<DeclKey> = None;
        let mut is_signed: Option<TypeSpec> = None;
        let mut int_cnt = 0;

        // 状态机循环
        for spec in specs {
            // 检查
            Self::check_type_spec(ctx, &spec, int_cnt, is_signed.as_ref());

            let next = match &spec.kind {
                Void => TypeSpecState::Void,
                Char => TypeSpecState::Char,
                Short => TypeSpecState::Short,
                Int => {
                    int_cnt += 1;
                    TypeSpecState::Int
                }
                Long => TypeSpecState::Long,
                Float => TypeSpecState::Float,
                Double => TypeSpecState::Double,
                Record(x) => {
                    decl = Some(*x);
                    TypeSpecState::Record
                }
                Enum(x) => {
                    decl = Some(*x);
                    TypeSpecState::Enum
                }
                TypeName(_, x) => {
                    decl = Some(*x);
                    TypeSpecState::TypeName
                }
                Signed | Unsigned => {
                    // 不参与循环
                    is_signed = Some(spec);
                    continue;
                }
            };

            state = match TypeSpecState::combine(state, next) {
                Some(x) => x,
                None => {
                    let err = ParserError::non_combinable(spec.to_string(), DECL_SPEC, spec.span);
                    return Err(err);
                }
            };
        }
        let is_signed = is_signed.map(|x| x.kind.is_signed()).unwrap_or(false);
        let builder_kind = Self::get_type_build_kind(ctx, state, is_signed, decl);

        Ok(builder_kind)
    }

    /// 构建 type builder kind, 不负责检查
    fn get_type_build_kind(
        ctx: &mut CompCtx,
        state: TypeSpecState,
        is_signed: bool,
        decl: Option<DeclKey>,
    ) -> TypeBuilderKind {
        use TypeSpecState::*;
        // 根据最后状态判断类型
        let builder = match state {
            Void => TypeBuilderKind::Void,
            Char => TypeBuilderKind::Integer {
                is_signed,
                size: IntegerSize::Char,
            },
            Short => TypeBuilderKind::Integer {
                is_signed,
                size: IntegerSize::Short,
            },
            Int => TypeBuilderKind::Integer {
                is_signed,
                size: IntegerSize::Int,
            },
            Long => TypeBuilderKind::Integer {
                is_signed,
                size: IntegerSize::Long,
            },
            LongLong => TypeBuilderKind::Integer {
                is_signed,
                size: IntegerSize::LongLong,
            },
            Float => TypeBuilderKind::Floating {
                size: FloatSize::Float,
            },
            Double => TypeBuilderKind::Floating {
                size: FloatSize::Double,
            },
            LongDouble => TypeBuilderKind::Floating {
                size: FloatSize::LongDouble,
            },
            Record => {
                let decl = decl.expect("record decl should not be none");
                let decl = ctx.get_decl(decl);
                let record_kind = match &decl.kind {
                    DeclKind::RecordDecl { kind, .. } => kind.kind,
                    DeclKind::RecordDef { kind, .. } => kind.kind,
                    _ => unreachable!(""),
                };
                TypeBuilderKind::new_record(ctx, record_kind)
            }
            Enum => TypeBuilderKind::new_enum(ctx),
            TypeName => {
                let decl = decl.expect("record decl should not be none");
                let decl = ctx.get_decl(decl);
                let ty = ctx.type_ctx.get_type(decl.ty);

                TypeBuilderKind::from_type_kind(&ty.kind)
            }
            Init => unreachable!("should not be init"),
        };
        builder
    }
}

// impl Sema {
//
//     /// 解析record的成员，插入decl
//     pub fn act_on_record_field(
//         &mut self,
//         ctx: &mut CompCtx,
//         struct_declarator: StructDeclarator,
//     ) -> ParserResult<DeclKey> {
//         let kind = DeclKind::RecordField {
//             colon: struct_declarator.colon,
//             bit_field: struct_declarator.bit_field,
//         };
//         let PartialDecl {
//             storage,
//             name,
//             ty_key: ty,
//         } = self.act_on_declarator(struct_declarator.declarator)?;
//         let decl = ctx.insert_decl(Decl {
//             storage,
//             name,
//             kind,
//             ty,
//             span: struct_declarator.span,
//         });
//         // 添加decl
//         self.insert_decl(decl)?;
//         Ok(decl)
//     }
//
//     /// 解析枚举成员，插入符号表
//     pub fn act_on_enumerator(
//         &mut self,
//         ctx: &mut CompCtx,
//         enumerator: Enumerator,
//     ) -> ParserResult<DeclKey> {
//         let kind = DeclKind::EnumField {
//             eq: enumerator.eq,
//             expr: enumerator.expr,
//         };
//         let ty = self.type_context.get_int_type(IntegerSize::Int, true);
//         let decl = ctx.insert_decl(Decl {
//             storage: None,
//             name: Some(enumerator.name),
//             kind,
//             ty,
//             span: enumerator.span,
//         });
//         // 添加decl
//         self.insert_decl(decl)?;
//         Ok(decl)
//     }
//
//     /// 类型参数
//     pub fn act_on_param_var(
//         &mut self,
//         ctx: &mut CompCtx,
//         declarator: Declarator,
//     ) -> ParserResult<DeclKey> {
//         let span = declarator.span;
//         let PartialDecl {
//             storage,
//             name,
//             ty_key: ty,
//         } = self.act_on_param_declarator(declarator)?;
//
//         let kind = DeclKind::ParamVar;
//         let decl = ctx.insert_decl(Decl {
//             storage,
//             name,
//             kind,
//             ty,
//             span,
//         });
//         Ok(decl)
//     }
//
//     /// 函数声明，添加函数声明和参数列表进入符号表
//     pub fn act_on_func_decl(
//         &mut self,
//         ctx: &mut CompCtx,
//         func_decl: FuncDecl,
//     ) -> ParserResult<DeclKey> {
//         let param = match func_decl.declarator.chunks.first() {
//             Some(x) => x,
//             None => {
//                 // 这不是函数声明
//                 todo!()
//             }
//         };
//
//         let param = match &param.kind {
//             DeclaratorChunkKind::Function { param, .. } => param,
//             _ => {
//                 // 不是函数，出错
//                 todo!()
//             }
//         };
//
//         let mut is_variadic = false;
//         let mut params = Vec::new();
//
//         match param {
//             ParamDecl::Params(x) => {
//                 // 普通param类型声明
//                 is_variadic = x.ellipsis.is_some();
//                 params.extend(x.params.iter().cloned());
//             }
//             ParamDecl::Idents(x) => {
//                 // K&R函数声明
//                 let decl = match &func_decl.decl_list {
//                     Some(x) => x,
//                     None => {
//                         // 这样一定出错
//                         todo!()
//                     }
//                 };
//
//                 let mut name_map: FxHashMap<Ident, DeclKey> = FxHashMap::default();
//
//                 let decls = decl.into_iter().map(|x| &x.decls).flatten().cloned();
//
//                 for x in decls {
//                     let decl = ctx.get_decl(x);
//                     let name = match decl.name.clone() {
//                         Some(x) => x,
//                         None => {
//                             // 不能没名字
//                             todo!()
//                         }
//                     };
//                     drop(decl);
//                     name_map.insert(name, x);
//                 }
//
//                 // 检查是否是一一对应
//                 for x in &x.idents {
//                     let decl = match name_map.remove(&x) {
//                         Some(x) => x,
//                         None => {
//                             // 没有对应的出错
//                             todo!()
//                         }
//                     };
//                     params.push(decl);
//                 }
//
//                 for (_, _decl) in name_map {
//                     // 存在不存在的声明，出错
//                     todo!()
//                 }
//             }
//         };
//
//         let PartialDecl {
//             storage,
//             name,
//             ty_key,
//         } = self.act_on_param_declarator(func_decl.declarator)?;
//         let mut decl_context = self.curr_decl.borrow_mut();
//
//         let ret_ty = match &ctx.get_type(ty_key).kind {
//             TypeKind::Function { ret_ty, .. } => ctx.get_type(*ret_ty),
//             _ => unreachable!(),
//         };
//
//         // 将参数压入context
//         for x in params.iter().copied() {
//             // 参数没名字，直接出错
//             if ctx.get_decl(x).name.is_none() {
//                 todo!()
//             }
//             decl_context.insert(ctx, x)?;
//         }
//         drop(decl_context);
//
//         let kind = DeclKind::FuncRef;
//
//         let decl = ctx.insert_decl(Decl {
//             storage,
//             name,
//             kind,
//             ty: ty_key,
//             span: func_decl.span,
//         });
//
//         self.insert_parent(decl)?;
//         Ok(decl)
//     }
//
//     /// 将声明插入符号表
//     pub fn act_on_record_ref(
//         &mut self,
//         ctx: &mut CompCtx,
//         record_kind: Record,
//         name: Ident,
//         span: Span,
//     ) -> ParserResult<()> {
//         let ty = self.type_context.resolve_record_ref(&record_kind, &name)?;
//         let kind = DeclKind::RecordRef { kind: record_kind };
//         let decl = ctx.insert_decl(Decl {
//             storage: None,
//             ty,
//             name: Some(name),
//             kind,
//             span,
//         });
//         self.insert_decl(decl)?;
//         Ok(())
//     }
//
//     /// 完成record声明或定义，会调用exit退出作用域 ,插入符号表
//     pub fn act_on_finish_record(
//         &mut self,
//         ctx: &mut CompCtx,
//         spec: StructSpec,
//     ) -> ParserResult<DeclKey> {
//         let decl_context = self.exit_decl();
//
//         let ty = self.type_context.resolve_record(&spec)?;
//
//         let kind = match spec.body {
//             None => DeclKind::RecordRef { kind: spec.kind },
//             Some(x) => DeclKind::Record {
//                 kind: spec.kind,
//                 l: x.l,
//                 fields: x.groups,
//                 r: x.r,
//                 decl_context,
//             },
//         };
//
//         let decl = ctx.insert_decl(Decl {
//             storage: None,
//             ty,
//             name: spec.name,
//             kind,
//             span: spec.span,
//         });
//
//         self.insert_decl(decl)?;
//         Ok(decl)
//     }
//
//     /// 完成enum声明/定义，会调用exit退出作用域 enum插入符号表
//     pub fn act_on_finish_enum(
//         &mut self,
//         ctx: &mut CompCtx,
//         spec: EnumSpec,
//     ) -> ParserResult<DeclKey> {
//         let decl_context = self.exit_decl();
//         let ty = self.type_context.resolve_enum(&spec)?;
//         let tkw = spec.enum_span;
//         let kind = match spec.enums {
//             None => DeclKind::EnumRef { kw },
//             Some(x) => DeclKind::Enum {
//                 kw,
//                 l: x.l,
//                 enums: x.decls,
//                 commas: x.commas,
//                 r: x.r,
//                 decl_context,
//             },
//         };
//
//         let decl = ctx.insert_decl(Decl {
//             storage: None,
//             ty,
//             name: spec.name,
//             kind,
//             span: spec.span,
//         });
//
//         self.insert_decl(decl)?;
//         Ok(decl)
//     }
//
//     /// 解析declarator
//     pub fn act_on_declarator(&mut self, declarator: Declarator) -> ParserResult<PartialDecl> {
//         let kind = self.curr_decl.borrow().get_kind();
//         let decl = match kind {
//             DeclContextKind::File => self.act_on_file_declarator(declarator)?,
//             DeclContextKind::Block => self.act_on_block_declarator(declarator)?,
//             DeclContextKind::Record => self.act_on_struct_declarator(declarator)?,
//             DeclContextKind::Enum => unreachable!(), // enum 内部没有 declarator
//         };
//
//         Ok(decl)
//     }
//
//     fn act_on_file_declarator(&mut self, declarator: Declarator) -> ParserResult<PartialDecl> {
//         // 默认extern
//         let storage = declarator
//             .decl_spec
//             .storage
//             .clone()
//             .unwrap_or(StorageSpec::from_kind(StorageSpecKind::Extern));
//         let name = declarator.name.clone();
//         let ty_key = self.type_context.resolve_declarator(&declarator)?;
//
//         // 顶级声明不能有auto和register
//         match &storage.kind {
//             StorageSpecKind::Auto | StorageSpecKind::Register => {
//                 todo!()
//             }
//             _ => {}
//         }
//
//         let result = PartialDecl {
//             storage: Some(storage),
//             name,
//             ty_key,
//         };
//         Ok(result)
//     }
//
//     fn act_on_block_declarator(&mut self, declarator: Declarator) -> ParserResult<PartialDecl> {
//         // 默认auto
//         let storage = declarator
//             .decl_spec
//             .storage
//             .clone()
//             .unwrap_or(StorageSpec::from_kind(StorageSpecKind::Auto));
//         let name = declarator.name.clone();
//         let ty = self.type_context.resolve_declarator(&declarator)?;
//
//         let result = PartialDecl {
//             storage: Some(storage),
//             name,
//             ty_key: ty,
//         };
//         Ok(result)
//     }
//
//     fn act_on_struct_declarator(&mut self, declarator: Declarator) -> ParserResult<PartialDecl> {
//         // 不允许任何storage声明
//         if declarator.decl_spec.storage.is_some() {
//             todo!()
//         }
//
//         let name = declarator.name.clone();
//         let ty = self.type_context.resolve_declarator(&declarator)?;
//
//         let result = PartialDecl {
//             storage: None,
//             name,
//             ty_key: ty,
//         };
//         Ok(result)
//     }
//
//     fn act_on_param_declarator(&mut self, declarator: Declarator) -> ParserResult<PartialDecl> {
//         let storage = declarator.decl_spec.storage.clone(); // 没有默认storage
//         let name = declarator.name.clone();
//         let ty = self.type_context.resolve_declarator(&declarator)?;
//
//         // storage只能是register
//         match &storage {
//             Some(x) => match x.kind {
//                 StorageSpecKind::Typedef
//                 | StorageSpecKind::Extern
//                 | StorageSpecKind::Static
//                 | StorageSpecKind::Auto => {
//                     todo!()
//                 }
//                 _ => {}
//             },
//             None => {}
//         }
//
//         let result = PartialDecl {
//             storage,
//             name,
//             ty_key: ty,
//         };
//         Ok(result)
//     }
// }
