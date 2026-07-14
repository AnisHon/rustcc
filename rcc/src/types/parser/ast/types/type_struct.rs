use crate::parser::ast::types::{CharSign, IntegerSign};
use crate::parser::comp_ctx::CompCtx;
use crate::types::parser::ast::common::RecordKind;
use crate::types::parser::ast::types::layout::TypeLayout;
use crate::types::parser::ast::types::primitives::{ArraySize, FloatingType, IntegerType};
use crate::types::parser::ast::types::qualifier::Qualifier;
use crate::types::parser::ast::{DeclKey, TypeKey};
use enum_as_inner::EnumAsInner;
use std::hash::Hash;
use std::sync::OnceLock;

/// 结构体ID
#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy)]
pub struct RecordID(pub usize);

/// 枚举ID
#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy)]
pub struct EnumID(pub usize);

pub enum TypeStatus {
    Complete,
    Incomplete,
}

/// 修饰类型结构
#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy)]
pub struct QualType {
    pub ty: TypeKey,
    pub qual: Qualifier,
}

impl QualType {
    pub fn new(ty: TypeKey, qual: Qualifier) -> Self {
        Self { ty, qual }
    }
}

impl From<TypeKey> for QualType {
    fn from(value: TypeKey) -> Self {
        QualType {
            ty: value,
            qual: Default::default(),
        }
    }
}

/// 裸类型结构
#[derive(Debug)]
pub struct Type {
    pub kind: TypeKind,
    layout: OnceLock<TypeLayout>,
}

impl Type {
    /// 创建类型使用默认的 Qual
    pub fn new(kind: TypeKind) -> Self {
        Self {
            kind,
            layout: OnceLock::new(),
        }
    }

    /// 获取 layout
    pub fn get_layout(&mut self, ctx: &CompCtx) -> &TypeLayout {
        self.layout.get_or_init(|| TypeLayout::new(ctx, self))
    }

    pub fn get_status(&mut self) -> TypeStatus {
        match &self.kind {
            TypeKind::BuildIn(_) => TypeStatus::Complete,
            TypeKind::Ptr(_) => TypeStatus::Complete,
            TypeKind::Array(x) => x.get_status(),
            TypeKind::Func(_) => TypeStatus::Complete,
            TypeKind::Tag(x) => x.get_status(),
            TypeKind::Error => TypeStatus::Incomplete,
        }
    }
}

/// 类型种类
/// # Variants
/// - `BuildIn`: 内建数据类型
/// - `Ptr`: 指针类型
/// - `Array`: 数组
/// - `Func`: 函数
/// - `Tag`: struct/union/enum
/// - `Error`: 错误类型
#[derive(Debug, Clone, EnumAsInner, Default)]
pub enum TypeKind {
    BuildIn(BuildInType),
    Ptr(PtrType),
    Array(ArrayType),
    Func(FuncType),
    Tag(TagType),
    #[default]
    Error,
}

impl TypeKind {
    pub fn new_array(elem_ty: QualType, size: ArraySize) -> Self {
        let array_type = ArrayType::new(elem_ty, size);
        Self::Array(array_type)
    }

    pub fn new_func(params: ParamsType, ret_ty: QualType) -> Self {
        let func = FuncType::new(params, ret_ty);
        Self::Func(func)
    }
    pub fn new_record(kind: RecordKind, id: RecordID, def: Option<DeclKey>) -> Self {
        let record = RecordType::new(kind, id, def);
        let tag_type = TagType::Record(record);
        Self::Tag(tag_type)
    }

    pub fn new_enum(id: EnumID, def: Option<DeclKey>) -> Self {
        let enum_ = EnumType::new(id, def);
        let tag_type = TagType::Enum(enum_);
        Self::Tag(tag_type)
    }
}

/// 内置类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BuildInType {
    Void,
    Bool, // 保留
    Char {
        sign: CharSign,
    },
    Integer {
        sign: IntegerSign,
        size: IntegerType,
    },
    Floating {
        size: FloatingType,
    },
    Complex {
        size: FloatingType,
    }, // 保留 负数
    Imaginary {
        size: FloatingType,
    }, // 保留 纯虚数
       // PtrdiffT, // 先去掉
       // SizeT,
}

/// 指针类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PtrType {
    pub elem_ty: QualType,
}

impl PtrType {
    pub fn new(elem_ty: QualType) -> Self {
        Self { elem_ty }
    }
}

/// 数组类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrayType {
    pub elem_ty: QualType,
    pub size: ArraySize,
}

impl ArrayType {
    pub fn new(elem_ty: QualType, size: ArraySize) -> Self {
        Self { elem_ty, size }
    }

    pub fn get_status(&self) -> TypeStatus {
        match self.size {
            ArraySize::VLA(_) | ArraySize::Static(_) => TypeStatus::Complete,
            ArraySize::Incomplete => TypeStatus::Incomplete,
        }
    }
}

/// 函数类型
/// # Members
/// - `params`: 参数列表
/// - `ret_ty`: 返回值
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FuncType {
    pub params: ParamsType,
    pub ret_ty: QualType,
}

impl FuncType {
    pub fn new(params: ParamsType, ret_ty: QualType) -> Self {
        Self { params, ret_ty }
    }
}

/// 参数类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParamsType {
    OldStyle, // 老式声明
    Prototype {
        // 原型声明，当params 为 None 时
        params: Option<Vec<QualType>>,
        is_variadic: bool,
    },
}

/// tag 类型
#[derive(Debug, Clone, EnumAsInner)]
pub enum TagType {
    Record(RecordType),
    Enum(EnumType),
}

impl TagType {}

impl TagType {
    pub fn get_status(&self) -> TypeStatus {
        match self {
            TagType::Record(x) => x.get_status(),
            TagType::Enum(x) => x.get_status(),
        }
    }
}

/// record 类型
/// - `kind`: record kind
/// - `id`: record 唯一标识符，所有对应的声明和定义共享
/// - `def`: 如果 None，代表 incomplete type
#[derive(Debug, Clone)]
pub struct RecordType {
    pub kind: RecordKind,
    pub id: RecordID,
    pub def: Option<DeclKey>,
}

impl RecordType {
    pub fn new(kind: RecordKind, id: RecordID, def: Option<DeclKey>) -> Self {
        Self { kind, id, def }
    }

    pub fn get_status(&self) -> TypeStatus {
        if self.def.is_some() {
            TypeStatus::Complete
        } else {
            TypeStatus::Incomplete
        }
    }
}

/// 枚举类型
/// - `id`: 枚举唯一标识符，所有对应的声明和定义共享
/// - `def`: 如果 None，代表 incomplete type
#[derive(Debug, Clone)]
pub struct EnumType {
    pub id: EnumID,
    pub def: Option<DeclKey>,
}

impl EnumType {
    pub fn new(id: EnumID, def: Option<DeclKey>) -> Self {
        Self { id, def }
    }

    pub fn get_status(&self) -> TypeStatus {
        if self.def.is_some() {
            TypeStatus::Complete
        } else {
            TypeStatus::Incomplete
        }
    }
}
