use crate::err::parser_error::ParserResult;
use crate::parser::ast::decl::DeclRef;
use crate::parser::semantic::ast::decl::{StructOrUnion, StructOrUnionKind};
use crate::parser::semantic::common::Ident;
use crate::parser::semantic::decl_spec::{DeclSpec, EnumSpec, ParamDecl, StructSpec, TypeQualKind, TypeQualType, TypeSpec, TypeSpecKind};
use crate::parser::semantic::declarator::{Declarator, DeclaratorChunkKind};
use crate::parser::semantic::sema::sema_type::{ArraySize, EnumField, FloatSize, IntegerSize, Qualifier, RecordField, Type, TypeKind};
use rustc_hash::FxHashSet;
use std::rc::Rc;

pub struct TypeContext {
    types: FxHashSet<Rc<Type>>,
}

impl TypeContext {
    pub fn new() -> Self {
        Self {
            types: FxHashSet::default(),
        }
    }

    /// 计算偏移量
    /// # Members
    /// - `offset`: 偏移量
    /// - `align`: 类型的对齐期望
    fn calc_offset(offset: u64, align: u64) -> u64 {
        // 计算需要的对齐填充
        let padding = (align - (offset % align)) % align;

        // 返回对齐后的偏移量
        offset + padding
    }

    /// 单例获取type
    pub fn get_or_set(&mut self, ty: Type) -> Rc<Type> {
        use TypeKind::*;
        match &ty.kind {
            // 普通的类型使用自身作为键
            Void | Unknown | Integer { .. } | Floating { .. }
            | Pointer { .. } | Array { .. } | Function { .. } => match self.types.get(&ty) {
                Some(x) => Rc::clone(x),
                None => {
                    let ty = Rc::new(ty);
                    self.types.insert(Rc::clone(&ty));
                    ty
                }
            }
            // 组合类型完全不用做单例
            Struct { .. } | StructRef { .. } | Union { .. }
            | UnionRef { .. } | Enum { .. } | EnumRef { .. } => {
                let ty = Rc::new(ty);
                self.types.insert(Rc::clone(&ty));
                ty
            }

        }

    }

    /// 解析qualifier const restrict violate
    fn resolve_qualifier(&mut self, qual: &TypeQualType) -> Qualifier {
        Qualifier {
            is_const: qual[TypeQualKind::Const as usize].is_some(),
            is_volatile: qual[TypeQualKind::Volatile as usize].is_some(),
            is_restrict: qual[TypeQualKind::Restrict as usize].is_some(),
        }
    }

    /// 解析type主体部分
    fn resolve_type_base(&mut self, specs: &[TypeSpec]) -> ParserResult<TypeKind> {
        let mut int = None;
        let mut signed = None;
        let mut state = TypeSpecState::Init;
        let mut decl: Option<_> = None;

        for spec in specs {
            let next_state = match &spec.kind {
                TypeSpecKind::Int => {
                    // 重复定义int
                    if int.is_some() {
                        todo!()
                    }
                    int = Some(spec);
                    TypeSpecState::Int
                }
                TypeSpecKind::Signed
                | TypeSpecKind::Unsigned => {
                    // 重复定义signed unsigned
                    if signed.is_some() {
                        todo!()
                    }
                    signed = Some(spec);
                    continue
                }
                TypeSpecKind::Void => TypeSpecState::Void,
                TypeSpecKind::Char => TypeSpecState::Char,
                TypeSpecKind::Short => TypeSpecState::Short,
                TypeSpecKind::Long => TypeSpecState::Long,
                TypeSpecKind::Float => TypeSpecState::Float,
                TypeSpecKind::Double => TypeSpecState::Double,
                TypeSpecKind::Struct(x) => {
                    decl = Some(x.borrow());
                    TypeSpecState::Struct
                },
                TypeSpecKind::Union(x) => {
                    decl = Some(x.borrow());
                    TypeSpecState::Union
                },
                TypeSpecKind::Enum(x) => {
                    decl = Some(x.borrow());
                    TypeSpecState::Enum
                },
                TypeSpecKind::TypeName(_, x) => {
                    decl = Some(x.borrow());
                    TypeSpecState::TypeName
                },
            };
            state = match combine(state, next_state) {
                Some(state) => state,
                None => {
                    // 转移失败
                    todo!();
                }
            };
        }

        let is_signed = signed.map(|x| x.kind.is_signed()).unwrap_or(false);

        // 查错
        match state {
            TypeSpecState::Float
            | TypeSpecState::Double
            | TypeSpecState::LongDouble
            | TypeSpecState::Struct
            | TypeSpecState::Union
            | TypeSpecState::Enum
            | TypeSpecState::TypeName => {
                // 不能组合signed unsigned
                if signed.is_some() {
                    todo!()
                }

                // 不能组合int
                if int.is_some() {
                    todo!()
                }
            }
            _ => {}
        }

        // 解析为TypeKind
        let kind = match state {
            TypeSpecState::Void => TypeKind::Void,
            TypeSpecState::Char => TypeKind::Integer { is_signed, size: IntegerSize::Char },
            TypeSpecState::Short => TypeKind::Integer { is_signed, size: IntegerSize::Short },
            TypeSpecState::Int => TypeKind::Integer { is_signed, size: IntegerSize::Int },
            TypeSpecState::Long => TypeKind::Integer { is_signed, size: IntegerSize::Long },
            TypeSpecState::LongLong => TypeKind::Integer { is_signed, size: IntegerSize::LongLong },
            TypeSpecState::Float => TypeKind::Floating { size: FloatSize::Float },
            TypeSpecState::Double => TypeKind::Floating { size: FloatSize::Double },
            TypeSpecState::LongDouble => TypeKind::Floating { size: FloatSize::LongDouble },
            TypeSpecState::Struct
            | TypeSpecState::Union
            | TypeSpecState::Enum
            | TypeSpecState::TypeName => decl.unwrap().ty.kind.clone(),
            _ => todo!() // todo 没有任何匹配
        };

        Ok(kind)
    }

    /// 解析decl_spec
    fn resolve_decl_spec(&mut self, decl_spec: &DeclSpec) -> ParserResult<Rc<Type>> {
        let qualifier = self.resolve_qualifier(&decl_spec.type_quals);
        let kind = self.resolve_type_base(&decl_spec.type_specs)?;
        let ty = Type::new(qualifier, kind);

        // 去重
        let ty = self.get_or_set(ty);
        Ok(ty)
    }

    /// 解析declarator
    pub fn resolve_declarator(&mut self, declarator: &Declarator) -> ParserResult<Rc<Type>> {
        let base_ty = self.resolve_decl_spec(&declarator.decl_spec)?;
        let mut ty = Rc::clone(&base_ty);
        // 解析chunks，这里一定要反着解析
        for chunk in declarator.chunks.iter().rev() {
            let new_ty = match &chunk.kind {
                DeclaratorChunkKind::Paren { .. } => {
                    // ignore
                    continue
                }
                DeclaratorChunkKind::Array { type_qual, expr, .. } => {
                    let qualifier = type_qual.as_ref()
                        .map(|x| self.resolve_qualifier(x))
                        .unwrap_or_default();
                    // 设置大小类型
                    let size = match expr {
                        None => ArraySize::Incomplete,
                        Some(x) => {
                            let expr_ty = &x.ty;
                            if x.ty.is_unknown() { 
                                todo!() // 类型未知
                            }

                            if !expr_ty.kind.is_integer() {
                                // todo 类型不对
                                todo!()
                            }

                            if x.is_int_constant() {
                                let sz = x.get_int_constant()?;
                                ArraySize::Static(sz)
                            } else {
                                ArraySize::VLA
                            }
                        }
                    };
                    let kind = TypeKind::Array { elem_ty: Rc::downgrade(&ty), size };
                    Type::new(qualifier, kind)
                }
                DeclaratorChunkKind::Pointer { type_qual, .. } => {
                    let qualifier = self.resolve_qualifier(type_qual);
                    let pointer = TypeKind::Pointer { elem_ty: Rc::downgrade(&ty) };
                    Type::new(qualifier, pointer)
                }
                DeclaratorChunkKind::Function { param, .. } => {
                    let list = match param {
                        ParamDecl::Params(list) => list,
                        ParamDecl::Idents(_) => todo!() // 声明不能用K&R参数
                    };
                    let is_variadic = list.ellipsis.is_some();
                    let params: Vec<_> = list.params.iter()
                        .cloned()
                        .map(|x| Rc::downgrade(&x.borrow().ty))
                        .collect();
                    let func = TypeKind::Function { 
                        ret_ty: Rc::downgrade(&ty),
                        params,
                        is_variadic,
                    };
                    Type::new(Qualifier::default(), func)
                }
            };

            ty = self.get_or_set(new_ty);
        }

        Ok(ty)
    }

    /// 不负责设置offset
    pub fn resolve_record_field(&mut self, decl: DeclRef) -> ParserResult<RecordField> {
        let field = decl.borrow();
        assert!(field.kind.is_record_field());
        let (_, bit_field) = field.kind.as_record_field().unwrap();
        let name  = field.name.clone();
        let ty = Rc::downgrade(&field.ty);
        let bit_field = match bit_field {
            None => None,
            Some(x) => Some(x.get_int_constant()?)
        };
        let field = RecordField {
            name,
            ty,
            bit_field,
            offset: 0,
        };
        Ok(field)
    }
    
    /// 解析record ref
    pub fn resolve_record_ref(&mut self, kind: &StructOrUnion, name: &Ident) -> ParserResult<Rc<Type>> {
        let name = name.clone();
        let kind = match kind.kind {
            StructOrUnionKind::Struct => TypeKind::StructRef { name },
            StructOrUnionKind::Union => TypeKind::UnionRef { name }
        };
        let ty = Rc::new(Type::new(Qualifier::default(), kind));
        Ok(ty)
    }
    
    /// 将StructSpec转换成Type
    pub fn resolve_record(&mut self, spec: &StructSpec) -> ParserResult<Rc<Type>> {
        let mut offset: u64 = 0;
        let mut fields = Vec::new();

        let body = match &spec.body {
            None => todo!(),
            Some(x) => x
        };
        for group in body.groups.iter() {
            for decl in group.decls.iter().cloned() {
                let mut field = self.resolve_record_field(decl)?;
                field.offset = offset;
                let align = match field.ty.upgrade().unwrap().align() {
                    None => todo!(),
                    Some(x) => x
                };
                offset = Self::calc_offset(offset, align);
                fields.push(field);
            }
        }

        let kind = match spec.kind.kind {
            StructOrUnionKind::Struct => TypeKind::Struct { name: spec.name.clone(), fields },
            StructOrUnionKind::Union => TypeKind::Union { name: spec.name.clone(), fields }
        };
        let ty = Rc::new(Type::new(Qualifier::default(), kind));
        Ok(ty)
    }

    /// 解析枚举类型
    pub fn resolve_enum(&mut self, spec: &EnumSpec) -> ParserResult<Rc<Type>> {
        let name = spec.name.clone();
        let mut enum_value = 0; // 当前枚举值

        let kind = match &spec.body {
            None => match name { // EnumRef
                Some(name) => TypeKind::EnumRef { name },
                None => todo!() // 引用不能没有名字
            }
            Some(body) => { // Enum定义
                let mut fields = Vec::new();
                for decl_ref in body.decls.iter() {
                    let decl = decl_ref.borrow();
                    // enum 声明一定有名字，expr必须是constant表达式
                    let name = decl.name.clone().unwrap();
                    let (_, expr) = decl.kind.as_enum_field().unwrap();

                    // 解析
                    let value = match expr {
                        None => enum_value, // 没写expr直接使用enum_value
                        Some(x) => x.get_int_constant()?
                    };
                    enum_value = value + 1; // enum_value指向下一个

                    let field = EnumField { name, value };
                    fields.push(field)
                }
                TypeKind::Enum { name, fields }
            }
        };

        let ty = Type::new(Qualifier::default(), kind);
        let ty = self.get_or_set(ty);
        Ok(ty)
    }
    
    pub fn get_int_type(&mut self, size: IntegerSize, is_signed: bool) -> Rc<Type> {
        let kind = TypeKind::Integer { size, is_signed };
        let ty = Type::new(Qualifier::default(), kind);
        self.get_or_set(ty)
    }

    /// 获取未知类型
    pub fn get_unknown_type(&mut self) -> Rc<Type> {
        let ty = Type { qual: Qualifier::default(), kind: TypeKind::Unknown };
        self.get_or_set(ty)
    }
}


/// 状态机状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypeSpecState {
    Init,
    Void,
    Char,
    Short,
    Int,
    Long,
    LongLong,
    Float,
    Double,
    LongDouble,
    Struct,
    Union,
    Enum,
    TypeName,
}

/// 类型转换定义
fn combine(state1: TypeSpecState, state2: TypeSpecState) -> Option<TypeSpecState> {
    use TypeSpecState::*;
    match (state1, state2) {
        (Init, _) => Some(state2),
        (Void, _) => None,
        (Char, Int) => Some(Char),
        (Short, Int) => Some(Short),
        (Int, Char) => Some(Char),
        (Int, Short) => Some(Short),
        (Int, Long) => Some(Int),
        (Int, LongLong) => Some(LongLong),
        (Long, Int) => Some(Long),
        (Long, Long) => Some(LongLong),
        (Long, Double) => Some(LongDouble),
        (LongLong, Int) => Some(LongLong),
        (Float, _) => None,
        (Double, Long) => Some(LongDouble),
        (LongDouble, _) => None,
        (Struct, _) => None,
        (Union, _) => None,
        (Enum, _) => None,
        (TypeName, _) => None,
        (_, _) => None,
    }
}

