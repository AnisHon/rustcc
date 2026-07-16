//! Declarator-facing C type structures.
//!
//! Parser builds this recursive representation because C declarator binding is naturally
//! recursive. After Sema succeeds, `TypeImporter` fills `CType::canonical` with the compact
//! QualType owned by the compilation's TypeContext.

use super::Expression;
use crate::source::SourceRange;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// C type qualifiers written on one declarator-facing type layer.
///
/// These booleans are convenient while parsing recursive declarators. They are
/// converted to the compact qualifier bitset in the canonical type system.
pub struct Qualifiers {
    pub is_const: bool,
    pub is_volatile: bool,
    pub is_restrict: bool,
    pub is_atomic: bool,
}

#[derive(Debug, Clone, PartialEq)]
/// Complete type information attached directly to a typed AST node.
///
/// `kind` retains inspectable declaration structure for clients, while
/// `canonical` provides cheap identity and compatibility queries after import.
pub struct CType {
    pub kind: TypeKind,
    pub qualifiers: Qualifiers,
    /// Canonical identity populated before the frontend returns `Compilation`.
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
/// Declarator-facing C type variants.
///
/// Record and enum definitions live inline here because this layer is the
/// public, inspectable AST. Their `TagId` still supplies nominal identity.
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
/// Nominal identity of one `struct`, `union`, or `enum` declaration.
///
/// Equal spellings in different scopes receive different IDs.
pub struct TagId(pub u32);

#[derive(Debug, Clone, PartialEq)]
/// The four array-bound forms distinguished by C declarator syntax.
pub enum ArraySize {
    /// A constant-size array such as `int a[4]`.
    Constant(usize),
    /// A variable length array whose bound is evaluated at runtime.
    Variable(Box<Expression>),
    /// An omitted bound, commonly completed by an initializer or prior declaration.
    Unspecified,
    /// The `[*]` form allowed in function prototype scope.
    Star,
}

#[derive(Debug, Clone, PartialEq)]
/// A record member, including anonymous members and optional bit-field width.
pub struct Field {
    pub name: Option<String>,
    pub ty: CType,
    pub bit_width: Option<u32>,
    pub range: SourceRange,
}

#[derive(Debug, Clone, PartialEq)]
/// One bound enumerator declaration and its computed integer value.
pub struct EnumVariant {
    pub id: super::DeclId,
    pub name: String,
    pub value: i64,
    pub range: SourceRange,
}

#[derive(Debug, Clone, PartialEq)]
/// A function parameter declaration after declarator adjustment and binding.
pub struct Parameter {
    pub id: super::DeclId,
    pub context: super::DeclContextId,
    pub name: Option<String>,
    pub ty: CType,
    pub range: SourceRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Valid normalized combinations of C storage-class specifiers.
///
/// Thread-local combinations are explicit variants so illegal combinations do
/// not leak into later semantic phases as independent flags.
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
/// Function-only declaration specifiers introduced or retained by C11.
pub struct FunctionSpecifiers {
    pub is_inline: bool,
    pub is_noreturn: bool,
}
