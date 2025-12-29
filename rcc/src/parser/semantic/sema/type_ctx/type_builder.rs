/// 构造Type的最小单位
use crate::{
    err::type_error::TypeError,
    parser::{
        ast::{
            TypeKey,
            common::RecordKind,
            types::{
                ArraySize, EnumID, FloatSize, IntegerSize, Qualifier, RecordID, Type, TypeKind,
            },
        },
        comp_ctx::CompCtx,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeBuilder {
    pub qual: Qualifier,
    pub kind: TypeBuilderKind,
}

impl TypeBuilder {
    pub fn new(kind: TypeBuilderKind) -> Self {
        Self {
            qual: Qualifier::default(),
            kind,
        }
    }

    pub fn new_with_qual(qual: Qualifier, kind: TypeBuilderKind) -> Self {
        Self { qual, kind }
    }

    /// 创建 integer 类型
    pub fn new_int(is_signed: bool, size: IntegerSize) -> Self {
        let kind = TypeBuilderKind::Integer { is_signed, size };
        Self::new(kind)
    }

    /// 创建 float 类型
    pub fn new_float(size: FloatSize) -> Self {
        let kind = TypeBuilderKind::Floating { size };
        Self::new(kind)
    }

    pub fn build(self) -> Result<Type, TypeError> {
        use TypeBuilderKind::*;
        let qual = self.qual;
        let kind = match self.kind {
            Void => TypeKind::Void,
            Unknown => TypeKind::Unknown,
            Integer { is_signed, size } => TypeKind::Integer { is_signed, size },
            Floating { size } => TypeKind::Floating { size },
            Pointer { elem_ty } => TypeKind::Pointer { elem_ty },
            Array { elem_ty, size } => TypeKind::Array { elem_ty, size },
            Function {
                ret_ty,
                params,
                is_variadic,
            } => TypeKind::Function {
                ret_ty,
                params,
                is_variadic,
            },
            Record { kind, id } => TypeKind::Record {
                kind,
                id,
                def: None,
            },
            Enum { id } => TypeKind::Enum { id, def: None },
        };
        let ty = Type::new_qual(qual, kind);
        Self::check_restrict(&ty)?;

        Ok(ty)
    }

    // 检查是否正确
    fn check_restrict(ty: &Type) -> Result<(), TypeError> {
        // 如果没用restrict直接忽略
        if !ty.qual.is_restrict {
            return Ok(());
        }

        todo!("实现restrict检查机制，只能用指针，指针指向的内容也要检查")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeBuilderKind {
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
    },
    Enum {
        id: EnumID, // C 的结构体用于区分 type identity, DeclKey 可以当作唯一标识符
    },
    Unknown, // 未知类型，用于出错后和初始化之类的
}

impl TypeBuilderKind {
    /// 构建一个全新的 record，分配一个 record id
    pub fn new_record(ctx: &mut CompCtx, kind: RecordKind) -> Self {
        let record_id = ctx.type_ctx.next_record_id();
        Self::Record {
            kind,
            id: record_id,
        }
    }

    /// 构建一个全新的 enum，分配一个 enum_id
    pub fn new_enum(ctx: &mut CompCtx) -> Self {
        let enum_id = ctx.type_ctx.next_enum_id();
        Self::Enum { id: enum_id }
    }

    pub fn from_type_kind(kind: &TypeKind) -> Self {
        match kind {
            TypeKind::Void => TypeBuilderKind::Void,
            TypeKind::Integer { is_signed, size } => TypeBuilderKind::Integer {
                is_signed: *is_signed,
                size: *size,
            },
            TypeKind::Floating { size } => TypeBuilderKind::Floating { size: *size },
            TypeKind::Pointer { elem_ty } => TypeBuilderKind::Pointer { elem_ty: *elem_ty },
            TypeKind::Array { elem_ty, size } => TypeBuilderKind::Array {
                elem_ty: *elem_ty,
                size: *size,
            },
            TypeKind::Function {
                ret_ty,
                params,
                is_variadic,
            } => TypeBuilderKind::Function {
                ret_ty: *ret_ty,
                params: *params,
                is_variadic: *is_variadic,
            },
            TypeKind::Record { id, kind, .. } => TypeBuilderKind::Record {
                kind: *kind,
                id: *id,
            },
            TypeKind::Enum { id, .. } => TypeBuilderKind::Enum { id: *id },
            TypeKind::Unknown => TypeBuilderKind::Unknown,
        }
    }
}
