use crate::parser::ast::common::RecordKind;
use crate::parser::ast::types::layout::TypeLayout;
use crate::parser::ast::types::primitives::{ArraySize, FloatSize, IntegerSize};
use crate::parser::ast::types::qualifier::Qualifier;
use crate::parser::ast::{DeclKey, TypeKey};
use crate::parser::comp_ctx::CompCtx;
use enum_as_inner::EnumAsInner;
use std::hash::Hash;
use std::sync::OnceLock;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy)]
pub struct RecordID(pub usize);
#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy)]
pub struct EnumID(pub usize);

#[derive(Debug, Clone, Default)]
pub struct Type {
    pub qual: Qualifier,
    pub kind: TypeKind,
    layout: OnceLock<TypeLayout>,
}

impl Type {
    /// 创建类型使用默认的 Qual
    pub fn new(kind: TypeKind) -> Self {
        Self {
            qual: Qualifier::default(),
            kind,
            layout: OnceLock::new(),
        }
    }

    /// 创建类型，使用外部的 Qual
    pub fn new_qual(qual: Qualifier, kind: TypeKind) -> Self {
        Self {
            qual,
            kind,
            layout: OnceLock::new(),
        }
    }

    /// 是否为完整类型，不完整类型是不能直接使用的（但是可以间接使用）
    /// `Void` `Unkown` 是绝对的不完整类型
    /// `Record` `Enum` 取决于是否有定义，纯声明是绝对的不完整类型
    /// 其余都是完整类型
    pub fn is_complete(&self) -> bool {
        use TypeKind::*;
        match &self.kind {
            Void | Unknown => false,
            Integer { .. } | Floating { .. } | Pointer { .. } | Array { .. } | Function { .. } => {
                true
            }
            Record { def, .. } => def.is_some(),
            Enum { def, .. } => def.is_some(),
        }
    }

    /// 获取 layout
    pub fn get_layout(&mut self, ctx: &CompCtx) -> &TypeLayout {
        self.layout.get_or_init(|| { TypeLayout::new(ctx, self) })
    }

}

#[derive(Debug, Clone, Default,EnumAsInner)]
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
    Record {
        kind: RecordKind,
        id: RecordID, // C 的结构体用于区分 type identity, DeclKey 可以当作唯一标识符
        def: Option<DeclKey>, // 如果definition 为 None，代表 incomplete type
    },
    Enum {
        id: EnumID,           // C 的结构体用于区分 type identity, DeclKey 可以当作唯一标识符
        def: Option<DeclKey>, // 如果definition 为 None，代表 incomplete type
    },
    #[default]
    Unknown, // 未知类型，用于出错后和初始化之类的
}
