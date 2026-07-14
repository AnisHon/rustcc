use std::fmt;

/// Compact identity of an interned, unqualified type.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TypeId(pub(crate) u32);

impl TypeId {
    pub const INVALID: Self = Self(u32::MAX);

    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

impl Default for TypeId {
    fn default() -> Self {
        Self::INVALID
    }
}

impl fmt::Debug for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TypeId({})", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Qualifiers(u8);

impl Qualifiers {
    pub const CONST: Self = Self(1 << 0);
    pub const VOLATILE: Self = Self(1 << 1);
    pub const RESTRICT: Self = Self(1 << 2);
    pub const ATOMIC: Self = Self(1 << 3);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QualType {
    pub ty: TypeId,
    pub qualifiers: Qualifiers,
}

impl Default for QualType {
    fn default() -> Self {
        Self::unqualified(TypeId::INVALID)
    }
}

impl QualType {
    pub const fn unqualified(ty: TypeId) -> Self {
        Self {
            ty,
            qualifiers: Qualifiers::empty(),
        }
    }

    pub const fn with_qualifiers(self, qualifiers: Qualifiers) -> Self {
        Self {
            ty: self.ty,
            qualifiers: self.qualifiers.union(qualifiers),
        }
    }

    pub const fn unqualified_type(self) -> Self {
        Self::unqualified(self.ty)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinType {
    Void,
    Bool,
    Char,
    SignedChar,
    UnsignedChar,
    Short,
    UnsignedShort,
    Int,
    UnsignedInt,
    Long,
    UnsignedLong,
    LongLong,
    UnsignedLongLong,
    Float,
    Double,
    LongDouble,
    FloatComplex,
    DoubleComplex,
    LongDoubleComplex,
    FloatImaginary,
    DoubleImaginary,
    LongDoubleImaginary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RecordId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnumId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecordKind {
    Struct,
    Union,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArrayBound {
    Constant(u64),
    Incomplete,
    Variable,
    Star,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CallingConvention {
    #[default]
    C,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionType {
    pub result: QualType,
    pub parameters: Vec<QualType>,
    pub variadic: bool,
    pub has_prototype: bool,
    pub calling_convention: CallingConvention,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeKind {
    Builtin(BuiltinType),
    Pointer(QualType),
    Array {
        element: QualType,
        bound: ArrayBound,
    },
    Function(FunctionType),
    Record {
        id: RecordId,
        kind: RecordKind,
    },
    Enum(EnumId),
    Atomic(QualType),
}
