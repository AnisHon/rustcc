use std::hash::{Hash, Hasher};
use std::rc::Rc;
use crate::parser::types::common::Ident;

///
/// # Members
/// - `name`: 成员名
/// - `ty`: 成员类型
/// - `bit_field`: 位域
/// - `offset`: 偏移量
///
#[derive(Debug, Clone)]
pub struct RecordField {
    pub name: Ident,
    pub ty: Rc<Type>,
    pub bit_field: Option<usize>,
    pub offset: usize,
}

impl PartialEq for RecordField {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && Rc::as_ptr(&self.ty) == Rc::as_ptr(&other.ty)
    }
}

impl Eq for RecordField {}

impl Hash for RecordField {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        Rc::as_ptr(&self.ty).hash(state);
    }
}

///
/// # Members
/// - `name`: 枚举名
/// - `value`: 枚举值
///
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumField {
    pub name: Ident,
    pub value: u64,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Qualifier {
    pub is_const: bool,
    pub is_volatile: bool,
    pub is_restrict: bool,
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    Void,
    Integer { signed: bool, size: IntegerSize },
    Floating { size: FloatSize },
    Pointer { elem_ty: Rc<Type> },
    Array { elem_ty: Rc<Type>, size: Option<i64> },
    Function { ret_ty: Rc<Type>, params: Vec<Type>, is_variadic: bool },
    Struct { name: Option<String>, fields: Vec<RecordField> },
    StructRef { name: Option<String>, },
    Union { name: Option<String>, fields: Vec<RecordField> },
    UnionRef { name: Option<String> },
    Enum { name: Option<String>, fields: Vec<EnumField> },
    EnumRef { name: Option<String> },
}

impl Hash for TypeKind {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use TypeKind::*;
        match self {
            Void => {
                0u8.hash(state);
            }
            Integer { signed, size } => {
                1u8.hash(state);
                signed.hash(state);
                size.hash(state);
            }
            Floating { size } => {
                2u8.hash(state);
                size.hash(state);
            }
            Pointer { elem_ty } => {
                3u8.hash(state);
                Rc::as_ptr(elem_ty).hash(state);
            }
            Array { elem_ty, size } => {
                4u8.hash(state);
                Rc::as_ptr(elem_ty).hash(state);
                size.hash(state);
            }
            Function { ret_ty, params, is_variadic } => {
                5u8.hash(state);
                Rc::as_ptr(ret_ty).hash(state);
                params.hash(state);
                is_variadic.hash(state);
            }
            StructRef { name } => {
                7u8.hash(state);
                name.hash(state);
            }
            UnionRef { name } => {
                9u8.hash(state);
                name.hash(state);
            }
            EnumRef { name } => {
                11u8.hash(state);
                name.hash(state);
            }
            Struct { name, fields } => {
                6u8.hash(state);
                name.hash(state);
                fields.hash(state);
            },
            Union { name, fields } => {
                8u8.hash(state);
                name.hash(state);
                fields.hash(state);
            }
            Enum { name, fields } => {
                10u8.hash(state);
                name.hash(state);
                fields.hash(state);
            }
        }
    }
}

impl PartialEq for TypeKind {
    fn eq(&self, other: &Self) -> bool {
        use TypeKind::*;
        match (self, other) {
            (Void, Void) => true,
            (
                Integer { signed: signed1, size: size1 },
                Integer { signed: signed2, size: size2 }
            ) => signed1 == signed2 && size1 == size2,
            (
                Floating { size: size1 },
                Floating { size: size2 }
            ) => size1 == size2,
            (
                Pointer { elem_ty: ty1 },
                Pointer { elem_ty: ty2 }
            ) => Rc::as_ptr(ty1) == Rc::as_ptr(ty2),
            (
                Array { elem_ty: ty1, size: size1 },
                Array { elem_ty: ty2, size: size2 }
            ) => Rc::as_ptr(ty1) == Rc::as_ptr(ty2) && size1 == size2,
            (
                Function { ret_ty: ty1, params: params1, is_variadic: variadic1 },
                Function { ret_ty: ty2, params: params2, is_variadic: variadic2 }
            ) => Rc::as_ptr(ty1) == Rc::as_ptr(ty2) && params1 == params2 && variadic1 == variadic2,
            (
                StructRef { name: name1 },
                StructRef { name: name2 },
            )
            | (
                UnionRef { name: name1 },
                UnionRef { name: name2 },
            )
            | (
                EnumRef { name: name1 },
                EnumRef { name: name2 },
            ) => name1 == name2,
            (
                Struct { name: name1, fields: fields1 },
                Struct { name: name2, fields: fields2 },
            ) 
            | (
                Union { name: name1, fields: fields1 },
                Union { name: name2, fields: fields2 }
            ) => name1 == name2 && fields1 == fields2,
            (
                Enum { name: name1, fields: fields1 },
                Enum { name: name2, fields: fields2 }
            ) => name1 == name2 && fields1 == fields2,
            (_, _) => false,
        }
    }
}

impl Eq for TypeKind {}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Type {
    pub qual: Qualifier,
    pub kind: TypeKind,
}

impl Type {
    pub fn new(qual: Qualifier, kind: TypeKind) -> Self {
        Self { qual, kind }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Hash)]
pub enum IntegerSize {
    Char,
    Short,
    Int,
    Long,
    LongLong,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Hash)]
pub enum FloatSize {
    Float,
    Double,
}