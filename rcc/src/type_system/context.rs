use super::{
    ArrayBound, BuiltinType, EnumId, FunctionType, QualType, RecordId, RecordKind, TypeId, TypeKind,
};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Construction failures for type combinations forbidden by C11.
///
/// Keeping these errors in the type-system module lets Sema attach source
/// ranges and wording without making `TypeContext` depend on diagnostics.
pub enum TypeError {
    ArrayOfFunction,
    ArrayOfVoid,
    FunctionReturnsArray,
    FunctionReturnsFunction,
    QualifiedFunction,
    AtomicArray,
    AtomicFunction,
    AtomicAtomic,
    AtomicQualified,
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::ArrayOfFunction => "array element type is a function type",
                Self::ArrayOfVoid => "array element type is void",
                Self::FunctionReturnsArray => "function return type is an array type",
                Self::FunctionReturnsFunction => "function return type is a function type",
                Self::QualifiedFunction => "function type cannot be qualified",
                Self::AtomicArray => "array type cannot be atomic",
                Self::AtomicFunction => "function type cannot be atomic",
                Self::AtomicAtomic => "atomic type cannot be applied to an atomic type",
                Self::AtomicQualified => "the operand of _Atomic cannot be qualified",
            }
        )
    }
}

impl std::error::Error for TypeError {}

/// Owns and canonicalizes all types for one compilation context.
#[derive(Debug)]
pub struct TypeContext {
    // `kinds` owns nodes; `interned` provides structural uniquing. A TypeId is an index into
    // `kinds`, so references between type nodes are compact and relocation-independent.
    kinds: Vec<TypeKind>,
    interned: HashMap<TypeKind, TypeId>,
    builtins: HashMap<BuiltinType, TypeId>,
    next_record: u32,
    next_enum: u32,
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeContext {
    pub fn new() -> Self {
        let mut context = Self {
            kinds: Vec::new(),
            interned: HashMap::new(),
            builtins: HashMap::new(),
            next_record: 0,
            next_enum: 0,
        };
        for builtin in [
            BuiltinType::Void,
            BuiltinType::Bool,
            BuiltinType::Char,
            BuiltinType::SignedChar,
            BuiltinType::UnsignedChar,
            BuiltinType::Short,
            BuiltinType::UnsignedShort,
            BuiltinType::Int,
            BuiltinType::UnsignedInt,
            BuiltinType::Long,
            BuiltinType::UnsignedLong,
            BuiltinType::LongLong,
            BuiltinType::UnsignedLongLong,
            BuiltinType::Float,
            BuiltinType::Double,
            BuiltinType::LongDouble,
            BuiltinType::FloatComplex,
            BuiltinType::DoubleComplex,
            BuiltinType::LongDoubleComplex,
            BuiltinType::FloatImaginary,
            BuiltinType::DoubleImaginary,
            BuiltinType::LongDoubleImaginary,
        ] {
            let id = context.intern(TypeKind::Builtin(builtin));
            context.builtins.insert(builtin, id);
        }
        context
    }

    pub fn kind(&self, id: TypeId) -> &TypeKind {
        &self.kinds[id.index()]
    }

    pub fn builtin(&self, builtin: BuiltinType) -> QualType {
        QualType::unqualified(self.builtins[&builtin])
    }

    pub fn pointer(&mut self, pointee: QualType) -> QualType {
        let id = self.intern(TypeKind::Pointer(pointee));
        QualType::unqualified(id)
    }

    pub fn array(&mut self, element: QualType, bound: ArrayBound) -> Result<QualType, TypeError> {
        match self.kind(element.ty) {
            TypeKind::Function(_) => return Err(TypeError::ArrayOfFunction),
            TypeKind::Builtin(BuiltinType::Void) => return Err(TypeError::ArrayOfVoid),
            _ => {}
        }
        let id = self.intern(TypeKind::Array { element, bound });
        Ok(QualType::unqualified(id))
    }

    pub fn function(&mut self, function: FunctionType) -> Result<QualType, TypeError> {
        match self.kind(function.result.ty) {
            TypeKind::Array { .. } => return Err(TypeError::FunctionReturnsArray),
            TypeKind::Function(_) => return Err(TypeError::FunctionReturnsFunction),
            _ => {}
        }
        let id = self.intern(TypeKind::Function(function));
        Ok(QualType::unqualified(id))
    }

    pub fn fresh_record(&mut self, kind: RecordKind) -> QualType {
        // Records are nominal C types. Never structurally merge two declarations, even if their
        // names and field layouts are identical.
        let id = RecordId(self.next_record);
        self.next_record += 1;
        QualType::unqualified(self.intern(TypeKind::Record { id, kind }))
    }

    pub fn fresh_enum(&mut self) -> QualType {
        let id = EnumId(self.next_enum);
        self.next_enum += 1;
        QualType::unqualified(self.intern(TypeKind::Enum(id)))
    }

    pub fn atomic(&mut self, value: QualType) -> Result<QualType, TypeError> {
        if value.qualifiers != super::Qualifiers::empty() {
            return Err(TypeError::AtomicQualified);
        }
        match self.kind(value.ty) {
            TypeKind::Array { .. } => return Err(TypeError::AtomicArray),
            TypeKind::Function(_) => return Err(TypeError::AtomicFunction),
            TypeKind::Atomic(_) => return Err(TypeError::AtomicAtomic),
            _ => {}
        }
        let id = self.intern(TypeKind::Atomic(value));
        Ok(QualType::unqualified(id))
    }

    /// C compatible types have the same canonical identity. Top-level qualifiers are ignored.
    pub fn compatible(&self, left: QualType, right: QualType) -> bool {
        left.ty == right.ty
    }

    pub fn canonical(&self, ty: QualType) -> QualType {
        ty
    }

    fn intern(&mut self, kind: TypeKind) -> TypeId {
        // Structural types (pointer/array/function/atomic) share one node whenever all operands
        // are identical. This makes canonical type comparison a TypeId comparison.
        if let Some(id) = self.interned.get(&kind) {
            return *id;
        }
        let id = TypeId(u32::try_from(self.kinds.len()).expect("too many types"));
        self.kinds.push(kind.clone());
        self.interned.insert(kind, id);
        id
    }
}
