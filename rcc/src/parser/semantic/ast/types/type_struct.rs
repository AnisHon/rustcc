use crate::lex::types::token_kind::Symbol;
use crate::parser::ast::types::primitives::{ArraySize, FloatSize, IntegerSize};
use crate::parser::ast::types::qualifier::Qualifier;
use crate::parser::ast::types::record::{RecordField};
use enum_as_inner::EnumAsInner;
use slotmap::new_key_type;
use std::hash::Hash;

new_key_type! {
    pub struct TypeKey;
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Type {
    pub qual: Qualifier,
    pub kind: TypeKind,
}

impl Type {

    /// 创建类型使用默认的 Qual
    pub fn new(kind: TypeKind) -> Self {
        Self {
            qual: Qualifier::default(),
            kind,
        }
    }

    /// 创建类型，使用外部的 Qual
    pub fn new_qual(qual: Qualifier, kind: TypeKind) -> Self {
        Self { qual, kind }
    }

    /// 创建 integer 类型
    pub fn new_int(is_signed: bool, size: IntegerSize) -> Self {
        let kind = TypeKind::Integer { is_signed, size };
        Self::new(kind)
    }

    /// 创建 float 类型
    pub fn new_float(size: FloatSize) -> Self {
        let kind = TypeKind::Floating { size };
        Self::new(kind)
    }

}

#[derive(Debug, Clone, EnumAsInner)]
pub enum TypeKind {
    Void,
    Integer {
        is_signed: bool,
        size: IntegerSize,
    },
    Floating {
        size: FloatSize,
    },
    Pointer {
        elem_ty: TypeKey,
    },
    Array {
        elem_ty: TypeKey,
        size: ArraySize,
    },
    Function {
        ret_ty: TypeKey,
        params: Vec<TypeKey>,
        is_variadic: bool,
    },
    Struct {
        name: Option<Symbol>,
        fields: Vec<RecordField>,
        size: u64, // 占用大小
    },
    StructRef {
        name: Symbol,
    },
    Union {
        name: Option<Symbol>,
        fields: Vec<RecordField>,
        size: u64, // 占用大小
    },
    UnionRef {
        name: Symbol,
    },
    Enum {
        name: Option<Symbol>,
    },
    EnumRef {
        name: Symbol,
    },
    Unknown, // 未知类型，用于出错后和初始化之类的
}
