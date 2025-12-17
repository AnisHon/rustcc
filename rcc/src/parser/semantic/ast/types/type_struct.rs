use crate::parser::ast::types::primitives::{ArraySize, FloatSize, IntegerSize};
use crate::parser::ast::types::qualifier::Qualifier;
use crate::parser::ast::types::record::{EnumField, RecordField};
use crate::parser::semantic::common::Ident;
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
    pub fn new(qual: Qualifier, kind: TypeKind) -> Self {
        Self { qual, kind }
    }
}

#[derive(Debug, Clone, EnumAsInner)]
pub enum TypeKind {
    Void,
    Integer { is_signed: bool, size: IntegerSize },
    Floating { size: FloatSize },
    Pointer { elem_ty: TypeKey },
    Array { elem_ty: TypeKey, size: ArraySize },
    Function {
        ret_ty: TypeKey,
        params: Vec<TypeKey>,
        is_variadic: bool,
    },
    Struct {
        name: Option<Ident>,
        fields: Vec<RecordField>,
        size: u64, // 占用大小
    },
    StructRef {
        name: Ident,
    },
    Union {
        name: Option<Ident>,
        fields: Vec<RecordField>,
        size: u64, // 占用大小
    },
    UnionRef {
        name: Ident
    },
    Enum {
        name: Option<Ident>,
        fields: Vec<EnumField>
    },
    EnumRef {
        name: Ident
    },
    Unknown // 未知类型，用于出错后和初始化之类的
}
