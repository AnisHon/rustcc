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
/// Compact top-level C qualifier set carried alongside a canonical `TypeId`.
///
/// Qualifiers are not part of the unqualified type identity, so `const int`
/// and `int` share a `TypeId` but have different `QualType` values.
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
/// Canonical C type reference: interned unqualified identity plus qualifiers.
///
/// This is the normal type handle exchanged by Sema and later compiler phases.
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
/// Fundamental arithmetic and `void` types supplied by the target C implementation.
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
/// Nominal identity of one canonical `struct` or `union` type.
pub struct RecordId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Nominal identity of one canonical enumeration type.
pub struct EnumId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Selects the distinct layout and member-overlap rules of records.
pub enum RecordKind {
    Struct,
    Union,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Canonical array-bound category; VLA expressions remain owned by the AST.
pub enum ArrayBound {
    Constant(u64),
    Incomplete,
    Variable,
    Star,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
/// Target calling convention attached to a function type.
///
/// Only standard C is modeled now; the enum leaves a controlled extension point
/// for target-specific conventions without changing function-type structure.
pub enum CallingConvention {
    #[default]
    C,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Canonical function signature after C parameter type adjustments.
pub struct FunctionType {
    pub result: QualType,
    pub parameters: Vec<QualType>,
    pub variadic: bool,
    pub has_prototype: bool,
    pub calling_convention: CallingConvention,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Interned type-system nodes.
///
/// Pointer, array, function, and atomic nodes are structurally uniqued. Record
/// and enum nodes contain nominal IDs and therefore never merge by layout.
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
