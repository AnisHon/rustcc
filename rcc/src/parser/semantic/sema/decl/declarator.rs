use crate::constant::str::DECL_SPEC;
use crate::err::parser_error::{ParserError, ParserResult};
use crate::parser::ast::types::{IntegerSize, TypeKind};
use crate::parser::common::TypeSpecState;
use crate::parser::comp_ctx::CompCtx;
use crate::parser::semantic::ast::decl::{Decl, DeclKind, Record};
use crate::parser::semantic::ast::func::FuncDecl;
use crate::parser::semantic::common::Ident;
use crate::parser::semantic::decl_spec::{
    DeclSpec, EnumSpec, Enumerator, FuncSpec, ParamDecl, StorageSpec, StorageSpecKind,
    StructDeclarator, StructSpec, TypeQual, TypeQualKind, TypeQuals, TypeSpec, TypeSpecKind,
};
use crate::parser::semantic::declarator::{Declarator, DeclaratorChunkKind, InitDeclarator};
use crate::parser::semantic::sema::Sema;
use crate::parser::semantic::sema::decl::decl_context::DeclContextKind;
use crate::parser::semantic::sema::type_ctx::type_builder::TypeBuilderKind;
use crate::types::span::Span;
use rustc_hash::FxHashMap;
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
            span,
        });

        Ok(decl_spec)
    }

    fn act_on_storages(storages: Vec<StorageSpec>) -> ParserResult<Option<StorageSpec>> {
        let mut storage: Option<StorageSpec> = None;
        for spec in storages {
            if let Some(x) = storage {
                let err = ParserError::duplicate(x.to_string(), DECL_SPEC, span);
                return Err(err);
            }
            storage = Some(spec);
        }

        Ok(stroage)
    }

    fn act_on_type_quals(quals: Vec<TypeQual>) -> ParserResult<TypeQuals> {
        use TypeQualKind::*;
        let res = TypeQuals::default();
        for qual in quals {
            let field = match qual.kind {
                Const => &mut res.is_const,
                Restrict => &mut res.is_restrict,
                Volatile => &mut res.is_volatile,
            };

            if let Some(x) = field {
                let err = ParserError::duplicate(x.to_string(), DECL_SPEC, span);
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
                let err = ParserError::duplicate(x.to_string(), DECL_SPEC, span);
                return Err(err);
            }
            func_spec = Some(spec);
        }

        Ok(stroage)
    }

    fn act_on_type_specs(ctx: &mut CompCtx, specs: Vec<TypeSpec>) -> ParserResult<TypeBuilderKind> {
        use TypeSpecKind::*;
        let mut state = TypeSpecState::Init;
        let mut decl: Option<DeclKey> = None;
        let mut is_signed: Option<TypeSpec> = None;
        let mut int_cnt = 0;
        let mut span = Span::default();

        for spec in specs {
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
                    if let Some(x) = is_signed {
                        // 相同重复了
                        if x.is(&spec.kind) {
                            let err = ParserError::duplicate(x.to_string(), DECL_SPEC, spec.span);
                            ctx.send_error(err);
                        } else {
                            let err =
                                ParserError::non_combinable(x.to_string(), DECL_SPEC, spec.span);
                            return Err(err);
                        }
                    }
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

            if int_cnt > 1 {
                let err = ParserError::non_combinable(spec.to_string(), DECL_SPEC, spec.span);
                return Err(err);
            }
            span = spec.span;
        }

        resolve_type_spec(state, decl, is_signed, int_cnt, span)
    }
}

impl Sema {
    pub fn act_on_init_declarator(
        &mut self,
        ctx: &mut CompCtx,
        init_declarator: InitDeclarator,
    ) -> ParserResult<DeclKey> {
        let declarator = init_declarator.declarator;
        // let storage = declarator.decl_spec.storage.clone();
        // let name = declarator.name.clone();
        let type_spec = declarator.decl_spec.type_specs.first().unwrap().clone();
        let PartialDecl {
            storage,
            name,
            ty_key,
        } = self.act_on_declarator(declarator)?;

        let ty = ctx.get_type(ty_key);

        let is_typedef = storage.as_ref().is_some_and(|x| x.kind.is_typedef());
        let kind = if is_typedef {
            if init_declarator.init.is_some() {
                // todo 对 typedef 初始化 错误
                todo!();
            }
            DeclKind::TypeDef
        } else {
            match ty.kind {
                TypeKind::Void
                | TypeKind::Integer { .. }
                | TypeKind::Floating { .. }
                | TypeKind::Pointer { .. }
                | TypeKind::Array { .. } => DeclKind::VarInit {
                    eq: init_declarator.eq,
                    init: init_declarator.init,
                },
                TypeKind::Function { .. } => DeclKind::FuncRef,
                TypeKind::Struct { .. } | TypeKind::StructRef { .. } => {
                    return Ok(type_spec.kind.into_struct().unwrap());
                }
                TypeKind::Union { .. } | TypeKind::UnionRef { .. } => {
                    return Ok(type_spec.kind.into_union().unwrap());
                }
                TypeKind::Enum { .. } | TypeKind::EnumRef { .. } => {
                    return Ok(type_spec.kind.into_enum().unwrap());
                }
                // TypeKind::Unknown => {}
                _ => todo!(),
            }
        };

        let decl = Decl {
            storage,
            name,
            kind,
            ty: ty_key,
            span: init_declarator.span,
        };

        let decl = ctx.insert_decl(decl);
        // 添加decl
        self.insert_decl(decl)?;
        // println!("\n{:?}\n\n", self.curr_decl);
        Ok(decl)
    }

    /// 解析record的成员，插入decl
    pub fn act_on_record_field(
        &mut self,
        ctx: &mut CompCtx,
        struct_declarator: StructDeclarator,
    ) -> ParserResult<DeclKey> {
        let kind = DeclKind::RecordField {
            colon: struct_declarator.colon,
            bit_field: struct_declarator.bit_field,
        };
        let PartialDecl {
            storage,
            name,
            ty_key: ty,
        } = self.act_on_declarator(struct_declarator.declarator)?;
        let decl = ctx.insert_decl(Decl {
            storage,
            name,
            kind,
            ty,
            span: struct_declarator.span,
        });
        // 添加decl
        self.insert_decl(decl)?;
        Ok(decl)
    }

    /// 解析枚举成员，插入符号表
    pub fn act_on_enumerator(
        &mut self,
        ctx: &mut CompCtx,
        enumerator: Enumerator,
    ) -> ParserResult<DeclKey> {
        let kind = DeclKind::EnumField {
            eq: enumerator.eq,
            expr: enumerator.expr,
        };
        let ty = self.type_context.get_int_type(IntegerSize::Int, true);
        let decl = ctx.insert_decl(Decl {
            storage: None,
            name: Some(enumerator.name),
            kind,
            ty,
            span: enumerator.span,
        });
        // 添加decl
        self.insert_decl(decl)?;
        Ok(decl)
    }

    /// 类型参数
    pub fn act_on_param_var(
        &mut self,
        ctx: &mut CompCtx,
        declarator: Declarator,
    ) -> ParserResult<DeclKey> {
        let span = declarator.span;
        let PartialDecl {
            storage,
            name,
            ty_key: ty,
        } = self.act_on_param_declarator(declarator)?;

        let kind = DeclKind::ParamVar;
        let decl = ctx.insert_decl(Decl {
            storage,
            name,
            kind,
            ty,
            span,
        });
        Ok(decl)
    }

    /// 函数声明，添加函数声明和参数列表进入符号表
    pub fn act_on_func_decl(
        &mut self,
        ctx: &mut CompCtx,
        func_decl: FuncDecl,
    ) -> ParserResult<DeclKey> {
        let param = match func_decl.declarator.chunks.first() {
            Some(x) => x,
            None => {
                // 这不是函数声明
                todo!()
            }
        };

        let param = match &param.kind {
            DeclaratorChunkKind::Function { param, .. } => param,
            _ => {
                // 不是函数，出错
                todo!()
            }
        };

        let mut is_variadic = false;
        let mut params = Vec::new();

        match param {
            ParamDecl::Params(x) => {
                // 普通param类型声明
                is_variadic = x.ellipsis.is_some();
                params.extend(x.params.iter().cloned());
            }
            ParamDecl::Idents(x) => {
                // K&R函数声明
                let decl = match &func_decl.decl_list {
                    Some(x) => x,
                    None => {
                        // 这样一定出错
                        todo!()
                    }
                };

                let mut name_map: FxHashMap<Ident, DeclKey> = FxHashMap::default();

                let decls = decl.into_iter().map(|x| &x.decls).flatten().cloned();

                for x in decls {
                    let decl = ctx.get_decl(x);
                    let name = match decl.name.clone() {
                        Some(x) => x,
                        None => {
                            // 不能没名字
                            todo!()
                        }
                    };
                    drop(decl);
                    name_map.insert(name, x);
                }

                // 检查是否是一一对应
                for x in &x.idents {
                    let decl = match name_map.remove(&x) {
                        Some(x) => x,
                        None => {
                            // 没有对应的出错
                            todo!()
                        }
                    };
                    params.push(decl);
                }

                for (_, _decl) in name_map {
                    // 存在不存在的声明，出错
                    todo!()
                }
            }
        };

        let PartialDecl {
            storage,
            name,
            ty_key,
        } = self.act_on_param_declarator(func_decl.declarator)?;
        let mut decl_context = self.curr_decl.borrow_mut();

        let ret_ty = match &ctx.get_type(ty_key).kind {
            TypeKind::Function { ret_ty, .. } => ctx.get_type(*ret_ty),
            _ => unreachable!(),
        };

        // 将参数压入context
        for x in params.iter().copied() {
            // 参数没名字，直接出错
            if ctx.get_decl(x).name.is_none() {
                todo!()
            }
            decl_context.insert(ctx, x)?;
        }
        drop(decl_context);

        let kind = DeclKind::FuncRef;

        let decl = ctx.insert_decl(Decl {
            storage,
            name,
            kind,
            ty: ty_key,
            span: func_decl.span,
        });

        self.insert_parent(decl)?;
        Ok(decl)
    }

    /// 将声明插入符号表
    pub fn act_on_record_ref(
        &mut self,
        ctx: &mut CompCtx,
        record_kind: Record,
        name: Ident,
        span: Span,
    ) -> ParserResult<()> {
        let ty = self.type_context.resolve_record_ref(&record_kind, &name)?;
        let kind = DeclKind::RecordRef { kind: record_kind };
        let decl = ctx.insert_decl(Decl {
            storage: None,
            ty,
            name: Some(name),
            kind,
            span,
        });
        self.insert_decl(decl)?;
        Ok(())
    }

    /// 完成record声明或定义，会调用exit退出作用域 ,插入符号表
    pub fn act_on_finish_record(
        &mut self,
        ctx: &mut CompCtx,
        spec: StructSpec,
    ) -> ParserResult<DeclKey> {
        let decl_context = self.exit_decl();

        let ty = self.type_context.resolve_record(&spec)?;

        let kind = match spec.body {
            None => DeclKind::RecordRef { kind: spec.kind },
            Some(x) => DeclKind::Record {
                kind: spec.kind,
                l: x.l,
                fields: x.groups,
                r: x.r,
                decl_context,
            },
        };

        let decl = ctx.insert_decl(Decl {
            storage: None,
            ty,
            name: spec.name,
            kind,
            span: spec.span,
        });

        self.insert_decl(decl)?;
        Ok(decl)
    }

    /// 完成enum声明/定义，会调用exit退出作用域 enum插入符号表
    pub fn act_on_finish_enum(
        &mut self,
        ctx: &mut CompCtx,
        spec: EnumSpec,
    ) -> ParserResult<DeclKey> {
        let decl_context = self.exit_decl();
        let ty = self.type_context.resolve_enum(&spec)?;
        let kw = spec.enum_span;
        let kind = match spec.enums {
            None => DeclKind::EnumRef { kw },
            Some(x) => DeclKind::Enum {
                kw,
                l: x.l,
                enums: x.decls,
                commas: x.commas,
                r: x.r,
                decl_context,
            },
        };

        let decl = ctx.insert_decl(Decl {
            storage: None,
            ty,
            name: spec.name,
            kind,
            span: spec.span,
        });

        self.insert_decl(decl)?;
        Ok(decl)
    }

    /// 解析declarator
    pub fn act_on_declarator(&mut self, declarator: Declarator) -> ParserResult<PartialDecl> {
        let kind = self.curr_decl.borrow().get_kind();
        let decl = match kind {
            DeclContextKind::File => self.act_on_file_declarator(declarator)?,
            DeclContextKind::Block => self.act_on_block_declarator(declarator)?,
            DeclContextKind::Record => self.act_on_struct_declarator(declarator)?,
            DeclContextKind::Enum => unreachable!(), // enum 内部没有 declarator
        };

        Ok(decl)
    }

    fn act_on_file_declarator(&mut self, declarator: Declarator) -> ParserResult<PartialDecl> {
        // 默认extern
        let storage = declarator
            .decl_spec
            .storage
            .clone()
            .unwrap_or(StorageSpec::from_kind(StorageSpecKind::Extern));
        let name = declarator.name.clone();
        let ty_key = self.type_context.resolve_declarator(&declarator)?;

        // 顶级声明不能有auto和register
        match &storage.kind {
            StorageSpecKind::Auto | StorageSpecKind::Register => {
                todo!()
            }
            _ => {}
        }

        let result = PartialDecl {
            storage: Some(storage),
            name,
            ty_key,
        };
        Ok(result)
    }

    fn act_on_block_declarator(&mut self, declarator: Declarator) -> ParserResult<PartialDecl> {
        // 默认auto
        let storage = declarator
            .decl_spec
            .storage
            .clone()
            .unwrap_or(StorageSpec::from_kind(StorageSpecKind::Auto));
        let name = declarator.name.clone();
        let ty = self.type_context.resolve_declarator(&declarator)?;

        let result = PartialDecl {
            storage: Some(storage),
            name,
            ty_key: ty,
        };
        Ok(result)
    }

    fn act_on_struct_declarator(&mut self, declarator: Declarator) -> ParserResult<PartialDecl> {
        // 不允许任何storage声明
        if declarator.decl_spec.storage.is_some() {
            todo!()
        }

        let name = declarator.name.clone();
        let ty = self.type_context.resolve_declarator(&declarator)?;

        let result = PartialDecl {
            storage: None,
            name,
            ty_key: ty,
        };
        Ok(result)
    }

    fn act_on_param_declarator(&mut self, declarator: Declarator) -> ParserResult<PartialDecl> {
        let storage = declarator.decl_spec.storage.clone(); // 没有默认storage
        let name = declarator.name.clone();
        let ty = self.type_context.resolve_declarator(&declarator)?;

        // storage只能是register
        match &storage {
            Some(x) => match x.kind {
                StorageSpecKind::Typedef
                | StorageSpecKind::Extern
                | StorageSpecKind::Static
                | StorageSpecKind::Auto => {
                    todo!()
                }
                _ => {}
            },
            None => {}
        }

        let result = PartialDecl {
            storage,
            name,
            ty_key: ty,
        };
        Ok(result)
    }
}
