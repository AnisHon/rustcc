#[derive(Debug, Clone)]
pub struct Qualifier {
    pub is_const: bool,
    pub is_volatile: bool,
    pub is_restrict: bool,
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    Void,
    Integer { signed: bool, size: IntegerSize },
    Floating { size: FloatSize },
    Pointer { elem_ty: Box<Type> },
    Array { elem_ty: Box<Type>, size: Option<i64> },
    Function { ret_ty: Box<Type>, params: Vec<Type>, is_variadic: bool },
    Struct { name: Option<String>, },
    Union { name: Option<String> },
    Enum { name: Option<String> },
}

#[derive(Debug, Clone)]
pub struct Type {
    pub qual: Qualifier,
    pub kind: TypeKind,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub enum IntegerSize {
    Char,
    Short,
    Int,
    Long,
    LongLong,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub enum FloatSize {
    Float,
    Double,
}