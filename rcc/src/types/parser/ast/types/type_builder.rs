/// 构造Type的最小单位
use crate::parser::ast::types::{
    ArraySize, ArrayType, BuildInType, FuncType, ParamsType, PtrType, QualType, Signedness, TagType,
};
/// 构造Type的最小单位
use crate::types::parser::ast::{
    common::RecordKind,
    types::{EnumID, FloatType, IntegerType, RecordID, Type, TypeKind},
};


#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum TypeBuilder {
    BuildIn(BuildInType),
    Ptr(PtrType),
    Array(ArrayType),
    Func(FuncType),
    Tag(TagTypeBuilder),
    #[default]
    Error,
}

impl TypeBuilder {
    /// 创建PTR
    pub fn new_ptr(elem_ty: QualType) -> Self {
        let ptr = PtrType::new(elem_ty);
        Self::Ptr(ptr)
    }

    pub fn new_array(elem_ty: QualType, size: ArraySize) -> Self {
        let array = ArrayType::new(elem_ty, size);
        Self::Array(array)
    }

    pub fn new_func(params: ParamsType, ret_ty: QualType) -> Self {
        let func = FuncType::new(params, ret_ty);
        Self::Func(func)
    }
    /// 创建 integer 类型
    pub fn new_int(is_signed: bool, size: IntegerType) -> Self {
        let build_in = BuildInType::Integer { is_signed, size };
        Self::BuildIn(build_in)
    }

    /// 创建 float 类型
    pub fn new_float(size: FloatType) -> Self {
        let build_in_type = BuildInType::Floating { size };
        Self::BuildIn(build_in_type)
    }

    pub fn new_char(signedness: Signedness) -> Self {
        let build_in_type = BuildInType::Char { signedness };
        Self::BuildIn(build_in_type)
    }

    pub fn build(self) -> Type {
        let kind = match self {
            TypeBuilder::BuildIn(x) => TypeKind::BuildIn(x),
            TypeBuilder::Ptr(x) => TypeKind::Ptr(x),
            TypeBuilder::Array(x) => TypeKind::Array(x),
            TypeBuilder::Func(x) => TypeKind::Func(x),
            TypeBuilder::Tag(x) => match x {
                TagTypeBuilder::Record { kind, id } => TypeKind::new_record(kind, id, None),
                TagTypeBuilder::Enum { id } => TypeKind::new_enum(id, None),
            },
            TypeBuilder::Error => TypeKind::Error,
        };
        Type::new(kind)
    }
}

impl From<&TypeKind> for TypeBuilder {
    fn from(value: &TypeKind) -> Self {
        match value {
            TypeKind::BuildIn(x) => TypeBuilder::BuildIn(x.clone()),
            TypeKind::Ptr(x) => TypeBuilder::Ptr(x.clone()),
            TypeKind::Array(x) => TypeBuilder::Array(x.clone()),
            TypeKind::Func(x) => TypeBuilder::Func(x.clone()),
            TypeKind::Tag(x) => {
                let tag = TagTypeBuilder::from(x);
                TypeBuilder::Tag(tag)
            }
            TypeKind::Error => TypeBuilder::Error,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TagTypeBuilder {
    Record {
        kind: RecordKind,
        id: RecordID, // C 的结构体用于区分 type identity, DeclKey 可以当作唯一标识符
    },
    Enum {
        id: EnumID, // C 的结构体用于区分 type identity, DeclKey 可以当作唯一标识符
    },
}

impl From<&TagType> for TagTypeBuilder {
    fn from(value: &TagType) -> Self {
        match value {
            TagType::Record(x) => TagTypeBuilder::Record {
                kind: x.kind,
                id: x.id,
            },
            TagType::Enum(x) => TagTypeBuilder::Enum { id: x.id },
        }
    }
}
