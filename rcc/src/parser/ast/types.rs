use super::Expression;
use crate::source::SourceRange;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Qualifiers {
    pub is_const: bool,
    pub is_volatile: bool,
    pub is_restrict: bool,
    pub is_atomic: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CType {
    pub kind: TypeKind,
    pub qualifiers: Qualifiers,
    pub canonical: crate::QualType,
}

impl CType {
    pub fn new(kind: TypeKind) -> Self {
        Self {
            kind,
            qualifiers: Qualifiers::default(),
            canonical: crate::QualType::default(),
        }
    }
    pub fn int() -> Self {
        Self::new(TypeKind::Int { signed: true })
    }
    pub fn uint() -> Self {
        Self::new(TypeKind::Int { signed: false })
    }
    pub fn void() -> Self {
        Self::new(TypeKind::Void)
    }
    pub fn pointer(to: CType) -> Self {
        Self::new(TypeKind::Pointer(Box::new(to)))
    }
    pub fn is_integer(&self) -> bool {
        matches!(
            self.kind,
            TypeKind::Bool
                | TypeKind::Char { .. }
                | TypeKind::Short { .. }
                | TypeKind::Int { .. }
                | TypeKind::Long { .. }
                | TypeKind::LongLong { .. }
                | TypeKind::Enum { .. }
        )
    }
    pub fn is_arithmetic(&self) -> bool {
        self.is_integer()
            || matches!(
                self.kind,
                TypeKind::Float
                    | TypeKind::Double
                    | TypeKind::LongDouble
                    | TypeKind::Complex(_)
                    | TypeKind::Imaginary(_)
            )
    }
    pub fn is_scalar(&self) -> bool {
        self.is_arithmetic() || matches!(self.kind, TypeKind::Pointer(_))
    }
    pub fn decay(&self) -> Self {
        match &self.kind {
            TypeKind::Array { element, .. } => CType::pointer((**element).clone()),
            TypeKind::Function { .. } => CType::pointer(self.clone()),
            _ => self.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    Void,
    Bool,
    Char {
        signed: Option<bool>,
    },
    Short {
        signed: bool,
    },
    Int {
        signed: bool,
    },
    Long {
        signed: bool,
    },
    LongLong {
        signed: bool,
    },
    Float,
    Double,
    LongDouble,
    Complex(Box<CType>),
    Imaginary(Box<CType>),
    Pointer(Box<CType>),
    Array {
        element: Box<CType>,
        size: ArraySize,
    },
    Function {
        return_type: Box<CType>,
        params: Vec<Parameter>,
        variadic: bool,
        has_prototype: bool,
    },
    Struct {
        id: TagId,
        name: Option<String>,
        fields: Option<Vec<Field>>,
    },
    Union {
        id: TagId,
        name: Option<String>,
        fields: Option<Vec<Field>>,
    },
    Enum {
        id: TagId,
        name: Option<String>,
        variants: Option<Vec<EnumVariant>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TagId(pub u32);

#[derive(Debug, Clone, PartialEq)]
pub enum ArraySize {
    Constant(usize),
    Variable(Box<Expression>),
    Unspecified,
    Star,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: Option<String>,
    pub ty: CType,
    pub bit_width: Option<u32>,
    pub range: SourceRange,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub id: super::DeclId,
    pub name: String,
    pub value: i64,
    pub range: SourceRange,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub id: super::DeclId,
    pub context: super::DeclContextId,
    pub name: Option<String>,
    pub ty: CType,
    pub range: SourceRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StorageClass {
    Typedef,
    Extern,
    Static,
    Auto,
    Register,
    ThreadLocal,
    StaticThreadLocal,
    ExternThreadLocal,
    #[default]
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FunctionSpecifiers {
    pub is_inline: bool,
    pub is_noreturn: bool,
}
