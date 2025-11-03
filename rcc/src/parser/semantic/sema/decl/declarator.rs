use crate::err::parser_error::ParserResult;
use crate::parser::semantic::ast::decl::{Decl, DeclKind, StructOrUnion};
use crate::parser::semantic::ast::func::FuncDecl;
use crate::parser::semantic::common::Ident;
use crate::parser::semantic::decl_spec::{EnumSpec, Enumerator, ParamDecl, StorageSpec, StorageSpecKind, StructDeclarator, StructSpec};
use crate::parser::semantic::declarator::{Declarator, DeclaratorChunkKind, InitDeclarator};
use crate::parser::semantic::sema::decl::decl_context::DeclContextKind;
use crate::parser::semantic::sema::sema_struct::DeclRef;
use crate::parser::semantic::sema::sema_type::{IntegerSize, Type, TypeKind};
use crate::parser::semantic::sema::Sema;
use crate::types::span::Span;
use rustc_hash::FxHashMap;
use std::rc::Rc;

pub struct PartialDecl {
    pub storage: Option<StorageSpec>,
    pub name: Option<Ident>,
    pub ty: Rc<Type>,
}

impl Sema {
    pub fn act_on_init_declarator(&mut self, declarator: InitDeclarator) -> ParserResult<DeclRef> {


        let PartialDecl {
            storage,
            name,
            ty
        } = self.act_on_declarator(declarator.declarator)?;

        let is_typedef = storage.as_ref().is_some_and(|x| x.kind.is_typedef());
        let kind = if is_typedef {
            if declarator.init.is_some() {
                // todo 对 typedef 初始化 错误
                todo!();
            }
            DeclKind::TypeDef
        } else {
            DeclKind::VarInit { eq: declarator.eq, init: declarator.init }
        };

        let decl = Decl {
            storage,
            name,
            kind,
            ty,
            span: declarator.span
        };

        let decl = Decl::new_ref(decl);
        // 添加decl
        self.insert_decl(Rc::clone(&decl))?;
        Ok(decl)
    }

    /// 解析record的成员，插入decl
    pub fn act_on_record_field(&mut self, struct_declarator: StructDeclarator) -> ParserResult<DeclRef> {
        let kind = DeclKind::RecordField { colon: struct_declarator.colon, bit_field: struct_declarator.bit_field };
        let PartialDecl {
            storage,
            name,
            ty
        } = self.act_on_declarator(struct_declarator.declarator)?;
        let decl = Decl::new_ref(Decl {
            storage,
            name,
            kind,
            ty,
            span: struct_declarator.span
        });
        // 添加decl
        self.insert_decl(Rc::clone(&decl))?;
        Ok(decl)
    }

    /// 解析枚举成员，插入符号表
    pub fn act_on_enumerator(&mut self, enumerator: Enumerator) -> ParserResult<DeclRef> {
        let kind = DeclKind::EnumField { eq: enumerator.eq, expr: enumerator.expr };
        let ty = self.type_context.get_int_type(IntegerSize::Int, true);
        let decl = Decl::new_ref(Decl {
            storage: None,
            name: Some(enumerator.name),
            kind,
            ty,
            span: enumerator.span
        });
        // 添加decl
        self.insert_decl(Rc::clone(&decl))?;
        Ok(decl)
    }

    /// 类型参数
    pub fn act_on_param_var(&mut self, declarator: Declarator) -> ParserResult<DeclRef> {
        let span = declarator.span;
        let PartialDecl {
            storage,
            name,
            ty
        } = self.act_on_param_declarator(declarator)?;

        let kind = DeclKind::ParamVar;
        let decl = Decl::new_ref(Decl { storage, name, kind, ty, span });
        Ok(decl)
    }
    
    /// 函数声明，添加函数声明和参数列表进入符号表
    pub fn act_on_func_decl(&mut self, func_decl: FuncDecl) -> ParserResult<DeclRef> {
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
            ParamDecl::Params(x) => { // 普通param类型声明
                is_variadic = x.ellipsis.is_some();
                params.extend(x.params.iter().cloned());
            }
            ParamDecl::Idents(x) => { // K&R函数声明
                let decl = match &func_decl.decl_list {
                    Some(x) => x,
                    None => {
                        // 这样一定出错
                        todo!()
                    }
                };

                let mut name_map: FxHashMap<Ident, DeclRef> = FxHashMap::default();

                let decls = decl.into_iter()
                    .map(|x| &x.decls)
                    .flatten()
                    .cloned();

                for x in decls {
                    let decl = x.borrow();
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
            ty,
        } = self.act_on_param_declarator(func_decl.declarator)?;
        let mut decl_context = self.curr_decl.borrow_mut();


        let ret_ty = match &ty.kind {
            TypeKind::Function { ret_ty, .. } => ret_ty.upgrade().unwrap(),
            _ => unreachable!()
        };

        // 将参数压入context
        for x in params.iter() {
            decl_context.insert(Rc::clone(x))?;
        }
        drop(decl_context);

        let kind = DeclKind::FuncRef {
            ret_ty,
            params,
            is_variadic,
        };

        let decl = Decl::new_ref(Decl {
            storage,
            name,
            kind,
            ty,
            span: func_decl.span,
        });

        self.insert_decl(Rc::clone(&decl))?;
        Ok(decl)
    }

    /// 将声明插入符号表
    pub fn act_on_record_ref(&mut self, record_kind: StructOrUnion, name: Ident, span: Span) -> ParserResult<()> {
        let ty = self.type_context.resolve_record_ref(&record_kind, &name)?;
        let kind = DeclKind::RecordRef {
            kind: record_kind,
        };
        let decl = Decl::new_ref(Decl {
            storage: None,
            ty,
            name: Some(name),
            kind,
            span,
        });
        self.insert_decl(Rc::clone(&decl))?;
        Ok(())
    }

    /// 完成record声明或定义，会调用exit退出作用域 ,插入符号表
    pub fn act_on_finish_record(&mut self, spec: StructSpec) -> ParserResult<DeclRef> {
        let decl_context = self.exit_decl();

        let ty = self.type_context.resolve_record(&spec)?;

        let kind = match spec.body {
            None => DeclKind::RecordRef { kind: spec.kind },
            Some(x) => DeclKind::Record {
                kind: spec.kind,
                l: x.l,
                fields: x.groups,
                r: x.r,
                decl_context
            }
        };

        let decl = Decl::new_ref(Decl {
            storage: None,
            ty,
            name: spec.name,
            kind, 
            span: spec.span
        });

        self.insert_decl(Rc::clone(&decl))?;
        Ok(decl)
    }

    /// 完成enum声明/定义，会调用exit退出作用域 enum插入符号表
    pub fn act_on_finish_enum(&mut self, spec: EnumSpec) -> ParserResult<DeclRef> {
        let decl_context = self.exit_decl();
        let ty = self.type_context.resolve_enum(&spec)?;
        let kw = spec.enum_span;
        let kind = match spec.body {
            None => DeclKind::EnumRef { kw },
            Some(x) => {
                DeclKind::Enum { kw, l: x.l, enums: x.decls, commas: x.commas, r: x.r, decl_context }
            }
        };

        let decl = Decl::new_ref(Decl {
            storage: None, 
            ty,
            name: spec.name,
            kind, 
            span: spec.span 
        });

        self.insert_decl(Rc::clone(&decl))?;
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
        let storage = declarator.decl_spec.storage.clone()
            .unwrap_or(StorageSpec::from_kind(StorageSpecKind::Extern));
        let name = declarator.name.clone();
        let ty = self.type_context.resolve_declarator(&declarator)?;

        // 顶级声明不能有auto和register
        match &storage.kind {
            StorageSpecKind::Auto | StorageSpecKind::Register => {
                todo!()
            }
            _ => {}
        }

        let result = PartialDecl { storage: Some(storage), name, ty };
        Ok(result)
    }

    fn act_on_block_declarator(&mut self, declarator: Declarator) -> ParserResult<PartialDecl> {
        // 默认auto
        let storage = declarator.decl_spec.storage.clone()
            .unwrap_or(StorageSpec::from_kind(StorageSpecKind::Auto));
        let name = declarator.name.clone();
        let ty = self.type_context.resolve_declarator(&declarator)?;

        let result = PartialDecl { storage: Some(storage), name, ty };
        Ok(result)
    }

    fn act_on_struct_declarator(&mut self, declarator: Declarator) -> ParserResult<PartialDecl> {
        // 不允许任何storage声明
        if declarator.decl_spec.storage.is_some() {
            todo!()
        }

        let name = declarator.name.clone();
        let ty = self.type_context.resolve_declarator(&declarator)?;

        let result = PartialDecl { storage: None, name, ty };
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
                }
            None => {}
        }

        let result = PartialDecl { storage, name, ty };
        Ok(result)
    }

}