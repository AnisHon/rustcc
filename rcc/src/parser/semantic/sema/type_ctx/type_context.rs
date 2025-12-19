use crate::err::parser_error::ParserResult;
use crate::parser::semantic::ast::decl::{StructOrUnion, StructOrUnionKind};
use crate::parser::semantic::common::Ident;
use crate::parser::semantic::decl_spec::{EnumSpec, StructSpec};
use rustc_hash::{FxHashMap};
use slotmap::SlotMap;
use std::rc::Rc;
use crate::lex::types::token_kind::{FloatSuffix, IntSuffix, LiteralKind};
use crate::parser::ast::types::{FloatSize, IntegerSize, Qualifier, Type, TypeKey, TypeKind};

pub struct TypeCtx {
    types: FxHashMap<Type, TypeKey>,
    pool: SlotMap<TypeKey, Type>,
}
impl TypeCtx {
    pub fn new() -> Self {
        Self {
            types: FxHashMap::default(),
            pool: SlotMap::with_key(),
        }
    }
    
    pub fn get_type(&self, key: TypeKey) -> &Type {
        self.pool.get(key).expect("Type Not Exist")
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
    pub fn get_or_set(&mut self, ty: Type) -> TypeKey {
        use TypeKind::*;
        match &ty.kind {
            // 普通的类型使用自身作为键
            Void | Unknown | Integer { .. } | Floating { .. }
            | Pointer { .. } | Array { .. } | Function { .. } => match self.types.get(&ty) {
                Some(x) => Rc::clone(x),
                None => {
                    let ty = Rc::new(ty);
                    self.types.insert();
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
    pub fn resolve_record_ref(&mut self, kind: &StructOrUnion, name: &Ident) -> ParserResult<TypeKey> {
        let name = name.clone();
        let kind = match kind.kind {
            StructOrUnionKind::Struct => TypeKind::StructRef { name },
            StructOrUnionKind::Union => TypeKind::UnionRef { name }
        };
        let ty = Rc::new(Type::new(Qualifier::default(), kind));
        Ok(ty)
    }
    
    /// 将StructSpec转换成Type
    pub fn resolve_record(&mut self, spec: &StructSpec) -> ParserResult<TypeKey> {
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

        let size = offset + fields.last().map(|x| x.offset).unwrap_or(1); // 结构体默认大小1字节

        let kind = match spec.kind.kind {
            StructOrUnionKind::Struct => TypeKind::Struct { name: spec.name.clone(), fields, size },
            StructOrUnionKind::Union => TypeKind::Union { name: spec.name.clone(), fields, size }
        };
        let ty = Rc::new(Type::new(Qualifier::default(), kind));
        Ok(ty)
    }

    /// 解析枚举类型
    pub fn resolve_enum(&mut self, spec: &EnumSpec) -> ParserResult<TypeKey> {
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

    pub fn get_void_type(&mut self) -> Rc<Type> {
        let kind = TypeKind::Void;
        let ty = Type::new(Qualifier::default(), kind);
        self.get_or_set(ty)
    }

    pub(crate) fn get_constant_type(&mut self, constant: &LiteralKind) -> Rc<Type> {
        match constant {
            LiteralKind::Integer { suffix, .. } => {
                 suffix.map(|x| match x {
                     IntSuffix::U => self.get_int_type(IntegerSize::Int, false),
                     IntSuffix::L => self.get_int_type(IntegerSize::Long, true),
                     IntSuffix::UL => self.get_int_type(IntegerSize::Long, false),
                     IntSuffix::LL => self.get_int_type(IntegerSize::LongLong, true),
                     IntSuffix::ULL => self.get_int_type(IntegerSize::LongLong, false),
                 }).unwrap_or_else(|| self.get_int_type(IntegerSize::Int, true))
            }
            LiteralKind::Float { suffix, .. } => {
                let size = suffix.map(|x| match x {
                    FloatSuffix::F => FloatSize::Float,
                    FloatSuffix::L => FloatSize::LongDouble,
                }).unwrap_or(FloatSize::Double);
                let kind = TypeKind::Floating { size };
                self.get_or_set(Type::new(Qualifier::default(), kind))
            }
            LiteralKind::Char { .. } => {
                self.get_int_type(IntegerSize::Char, false)
            }
            LiteralKind::String { .. } => {
                let char_ty = self.get_int_type(IntegerSize::Char, false);
                let kind = TypeKind::Pointer { elem_ty: Rc::downgrade(&char_ty) };
                self.get_or_set(Type::new(Qualifier::default(), kind))
            }
        }
    }

    /// 获取未知类型
    pub fn get_unknown_type(&mut self) -> Rc<Type> {
        let ty = Type { qual: Qualifier::default(), kind: TypeKind::Unknown };
        self.get_or_set(ty)
    }
}

