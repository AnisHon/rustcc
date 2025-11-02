use crate::parser::types::common::Ident;
use enum_as_inner::EnumAsInner;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

///
/// # Members
/// - `name`: 成员名
/// - `ty`: 成员类型
/// - `bit_field`: 位域
/// - `offset`: 偏移量
///
#[derive(Debug, Clone)]
pub struct RecordField {
    pub name: Option<Ident>,
    pub ty: Rc<Type>,
    pub bit_field: Option<u64>,
    pub offset: u64,
}

impl RecordField {
    pub fn to_code(&self) -> String {
        let mut code = String::new();

        let ty = self.ty.to_code();
        let name = self.name.as_ref().map(|x| x.symbol.get()).unwrap_or_default();

        code.push_str(&ty);
        code.push(' ');
        code.push_str(name);

        match self.bit_field.map(|x| x.to_string()) {
            None => {},
            Some(x) => {
                code.push_str(": ");
                code.push_str(&x);
            }
        }

        code.push(';');
        code
    }
}

impl PartialEq for RecordField {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && Rc::as_ptr(&self.ty) == Rc::as_ptr(&other.ty)
    }
}

impl Eq for RecordField {}

impl Hash for RecordField {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        Rc::as_ptr(&self.ty).hash(state);
    }
}

///
/// # Members
/// - `name`: 枚举名
/// - `value`: 枚举值
///
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumField {
    pub name: Ident,
    pub value: u64,
}

impl EnumField {
    pub fn to_code(&self) -> String {
        let mut code = String::new();
        code.push_str(self.name.symbol.get());
        code.push('=');
        code.push_str(&self.value.to_string());
        code
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Qualifier {
    pub is_const: bool,
    pub is_volatile: bool,
    pub is_restrict: bool,
}

impl Qualifier {
    pub fn to_code(&self) -> String {
        let mut content = String::new();
        if self.is_const {
            content.push_str("const ")
        }

        if self.is_volatile {
            content.push_str("volatile ")
        }

        if self.is_restrict {
            content.push_str("restrict ")
        }

        content
    }
}

impl Default for Qualifier {
    fn default() -> Self {
        Self {
            is_const: false,
            is_volatile: false,
            is_restrict: false,
        }
    }
}

#[derive(Debug, Clone, EnumAsInner)]
pub enum TypeKind {
    Void,
    Integer { is_signed: bool, size: IntegerSize },
    Floating { size: FloatSize },
    Pointer { elem_ty: Rc<Type> },
    Array { elem_ty: Rc<Type>, size: ArraySize },
    Function { 
        ret_ty: Rc<Type>, 
        params: Vec<Rc<Type>>, 
        is_variadic: bool,
    },
    Struct { 
        name: Option<Ident>, 
        fields: Vec<RecordField> 
    },
    StructRef { 
        name: Ident, 
    },
    Union { 
        name: Option<Ident>, 
        fields: Vec<RecordField> 
    },
    UnionRef { 
        name: Ident 
    },
    Enum { 
        name: Option<Ident>, 
        fields: Vec<EnumField> 
    },
    EnumRef { 
        name: Ident 
    },
    Unknown // 未知类型，用于出错后和初始化之类的
}

impl Hash for TypeKind {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use TypeKind::*;
        match self {
            Void => {
                0u8.hash(state);
            }
            Integer { is_signed: signed, size } => {
                1u8.hash(state);
                signed.hash(state);
                size.hash(state);
            }
            Floating { size } => {
                2u8.hash(state);
                size.hash(state);
            }
            Pointer { elem_ty } => {
                3u8.hash(state);
                Rc::as_ptr(elem_ty).hash(state);
            }
            Array { elem_ty, size } => {
                4u8.hash(state);
                Rc::as_ptr(elem_ty).hash(state);
                size.hash(state);
            }
            Function { ret_ty, params, is_variadic, .. } => {
                5u8.hash(state);
                Rc::as_ptr(ret_ty).hash(state);
                params.hash(state);
                is_variadic.hash(state);
            }
            StructRef { name } => {
                7u8.hash(state);
                name.hash(state);
            }
            UnionRef { name } => {
                9u8.hash(state);
                name.hash(state);
            }
            EnumRef { name } => {
                11u8.hash(state);
                name.hash(state);
            }
            Struct { name, fields } => {
                6u8.hash(state);
                name.hash(state);
                fields.hash(state);
            },
            Union { name, fields } => {
                8u8.hash(state);
                name.hash(state);
                fields.hash(state);
            }
            Enum { name, fields } => {
                10u8.hash(state);
                name.hash(state);
                fields.hash(state);
            },
            Unknown => 12u8.hash(state),
        }
    }
}

impl PartialEq for TypeKind {
    fn eq(&self, other: &Self) -> bool {
        use TypeKind::*;
        match (self, other) {
            (Void, Void) | (Unknown, Unknown) => true,
            (
                Integer { is_signed: signed1, size: size1 },
                Integer { is_signed: signed2, size: size2 }
            ) => signed1 == signed2 && size1 == size2,
            (
                Floating { size: size1 },
                Floating { size: size2 }
            ) => size1 == size2,
            (
                Pointer { elem_ty: ty1 },
                Pointer { elem_ty: ty2 }
            ) => Rc::as_ptr(ty1) == Rc::as_ptr(ty2),
            (
                Array { elem_ty: ty1, size: size1 },
                Array { elem_ty: ty2, size: size2 }
            ) => Rc::as_ptr(ty1) == Rc::as_ptr(ty2) && size1 == size2,
            (
                Function { ret_ty: ty1, params: params1, is_variadic: variadic1, .. },
                Function { ret_ty: ty2, params: params2, is_variadic: variadic2, .. }
            ) => Rc::as_ptr(ty1) == Rc::as_ptr(ty2) && params1 == params2 && variadic1 == variadic2,
            (
                StructRef { name: name1 },
                StructRef { name: name2 },
            )
            | (
                UnionRef { name: name1 },
                UnionRef { name: name2 },
            )
            | (
                EnumRef { name: name1 },
                EnumRef { name: name2 },
            ) => name1 == name2,
            (
                Struct { name: name1, fields: fields1 },
                Struct { name: name2, fields: fields2 },
            ) 
            | (
                Union { name: name1, fields: fields1 },
                Union { name: name2, fields: fields2 }
            ) => name1 == name2 && fields1 == fields2,
            (
                Enum { name: name1, fields: fields1 },
                Enum { name: name2, fields: fields2 }
            ) => name1 == name2 && fields1 == fields2,
            (_, _) => false,
        }
    }
}

impl Eq for TypeKind {}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Type {
    pub qual: Qualifier,
    pub kind: TypeKind,
}

impl Type {
    pub fn new(qual: Qualifier, kind: TypeKind) -> Self {
        Self { qual, kind }
    }

    pub fn align(&self) -> Option<u64> {
        use TypeKind::*;
        match &self.kind {
            Void | Unknown  => None,
            Integer{ .. } => Some(4),
            Floating{ .. } => Some(4),
            Pointer{ .. } => Some(8),
            Array{ size, elem_ty } => elem_ty.align().map(|x| x * size.get_static()),
            Function{ .. } => Some(8),
            Struct{ fields, .. } => Some(fields.iter().map(|x| x.ty.align().unwrap()).sum()),
            StructRef{ .. } => None,
            Union{ fields, .. } => fields.iter().map(|x| x.ty.align().unwrap()).max(),
            UnionRef{ .. } => None,
            Enum{ .. } => Some(8),
            EnumRef{ .. } => None,
        }
    }
    
    pub fn is_unknown(&self) -> bool {
        matches!(&self.kind, TypeKind::Unknown)
    }

    pub fn to_code(&self) -> String {
        let mut code = String::new();
        let qual = self.qual.to_code();

        code.push_str(&qual);

        match &self.kind {
            TypeKind::Void => code.push_str("void "),
            TypeKind::Integer{ is_signed, size } => {
                if *is_signed {
                    code.push_str("signed ");
                } else {
                    code.push_str("unsigned ");
                }
                code.push_str(size.to_code());
                code.push(' ');
            },
            TypeKind::Floating{ size } => {
                code.push_str(size.to_code());
                code.push(' ');
            }
            TypeKind::Pointer{ elem_ty } => {
                code.push('*');
                code.push_str(&elem_ty.to_code());
                code.push(' ');
            }
            TypeKind::Array{ size, elem_ty } => {
                code.push_str(&elem_ty.to_code());
                code.push_str(&size.to_code());
                code.push(' ');
            }
            TypeKind::Function{ ret_ty, params, is_variadic } => {
                code.push_str("fn ");
                let param = params.iter().map(|x| x.to_code()).collect::<Vec<_>>().join(",");
                let variadic = is_variadic.then(|| ",...").unwrap_or_default();
                code.push_str(&format!("({}{})", param, variadic));
                code.push_str(" -> ");
                code.push_str(&ret_ty.to_code());
                code.push(' ');
            }
            TypeKind::Struct{ name, fields } => {
                let name = name.as_ref().map(|x| x.symbol.get()).unwrap_or_default();
                let fields: String = fields.iter().map(|x| x.to_code()).collect();
                code.push_str("struct ");
                code.push_str(name);
                code.push('{');
                code.push_str(&fields);
                code.push('}');
                code.push(' ');
            }
            TypeKind::StructRef{ name } => {
                code.push_str("struct ");
                code.push_str(name.symbol.get());
            }
            TypeKind::Union{ name, fields } => {
                let name = name.as_ref().map(|x| x.symbol.get()).unwrap_or_default();
                let fields: String = fields.iter().map(|x| x.to_code()).collect();
                code.push_str("union ");
                code.push_str(name);
                code.push('{');
                code.push_str(&fields);
                code.push('}');
                code.push(' ');
            }
            TypeKind::UnionRef{ name } => {
                code.push_str("union ");
                code.push_str(name.symbol.get());
            }
            TypeKind::Enum{ name, fields } => {
                let name = name.as_ref().map(|x| x.symbol.get()).unwrap_or_default();
                let fields = fields.iter()
                    .map(|x| x.to_code())
                    .collect::<Vec<_>>()
                    .join(",");
                code.push_str("union ");
                code.push_str(name);
                code.push('{');
                code.push_str(&fields);
                code.push('}');
                code.push(' ');
            }
            TypeKind::EnumRef{ name } => {
                code.push_str("enum ");
                code.push_str(name.symbol.get());
            }
            TypeKind::Unknown => {
                code.push_str("$ERROR$");
            }
        }


        code
    }
}

impl Default for Type {
    fn default() -> Self {
        Self {
            qual: Qualifier::default(),
            kind: TypeKind::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Hash)]
pub enum IntegerSize {
    Char,
    Short,
    Int,
    Long,
    LongLong,
}

impl IntegerSize {
    pub fn to_code(self) -> &'static str {
        match self {
            IntegerSize::Char => "char",
            IntegerSize::Short => "short",
            IntegerSize::Int => "int",
            IntegerSize::Long => "long",
            IntegerSize::LongLong => "long long"
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Hash)]
pub enum FloatSize {
    Float,
    Double,
    LongDouble,
}

impl FloatSize {
    pub fn to_code(self) -> &'static str {
        match self {
            FloatSize::Float => "float",
            FloatSize::Double => "double",
            FloatSize::LongDouble => "long double",
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ArraySize {
    Static(u64),    // int a[10]
    VLA, // int a[var]
    Incomplete,     // int a[]
}

impl ArraySize {
    pub fn get_static(&self) -> u64 {
        match self {
            ArraySize::Static(x) => *x,
            _ => unreachable!()
        }
    }

    pub fn to_code(&self) -> String {
        match self {
            ArraySize::Static(x) => format!("[{}]", x),
            ArraySize::VLA => "[]".to_owned(),
            ArraySize::Incomplete => "[]".to_owned(),
        }
    }
}