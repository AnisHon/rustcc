use crate::constant::str::DECL_SPEC;
use crate::err::parser_error::{ParserError, ParserResult};
use crate::parser::ast::DeclKey;
use crate::parser::ast::decl::DeclKind;
use crate::parser::ast::types::{FloatSize, IntegerSize};
use crate::parser::common::TypeSpecState;
use crate::parser::comp_ctx::CompCtx;
use crate::parser::semantic::decl_spec::{
    DeclSpec, FuncSpec, StorageSpec, TypeQual, TypeQuals, TypeSpec,
};
use crate::parser::semantic::sema::type_ctx::type_builder::TypeBuilderKind;
use crate::types::span::Span;
use std::rc::Rc;

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
        use crate::parser::semantic::decl_spec::TypeQualKind::*;
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
        use crate::parser::semantic::decl_spec::TypeSpecKind::*;
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
        use crate::parser::semantic::decl_spec::TypeSpecKind::*;
        debug_assert!(!specs.is_empty());
        let mut state = TypeSpecState::Init;
        let mut decl: Option<DeclKey> = None;
        let mut is_signed: Option<TypeSpec> = None;
        let mut int_cnt = 0;

        // 状态机循环
        for spec in specs {
            // 检查
            Self::check_type_spec(ctx, &spec, int_cnt, is_signed.as_ref())?;

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
        use crate::parser::semantic::common::TypeSpecState::*;
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
