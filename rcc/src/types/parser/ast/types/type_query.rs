use crate::parser::ast::common::RecordKind;
use crate::parser::ast::types::{BuildInType, TagType, Type, TypeKind};

impl Type {
    /// `short`, `int`, `long`, `long long`, `char`, `_Bool`, `enum`
    pub fn is_integral_type(&self) -> bool {
        match &self.kind {
            TypeKind::BuildIn(x) => match x {
                BuildInType::Bool | BuildInType::Char { .. } | BuildInType::Integer { .. } => true,
                BuildInType::Floating { .. }
                | BuildInType::Complex { .. }
                | BuildInType::Imaginary { .. }
                | BuildInType::Void => false,
            },
            TypeKind::Tag(x) => match x {
                TagType::Record(x) => false,
                TagType::Enum(x) => true,
            },
            _ => false,
        }
    }

    /// `float`, `double`, `long double`, `float _Complex`, `double _Complex`, `long double _Complex`
    /// `float _Imaginary`, `double _Imaginary`, `long double _Imaginary`
    pub fn is_floating_point_type(&self) -> bool {
        match &self.kind {
            TypeKind::BuildIn(x) => match x {
                BuildInType::Bool | BuildInType::Char { .. } | BuildInType::Integer { .. } => false,
                BuildInType::Floating { .. }
                | BuildInType::Complex { .. }
                | BuildInType::Imaginary { .. }
                | BuildInType::Void => true,
            },
            _ => false,
        }
    }

    /// integral types or floating point types
    pub fn is_arithmetic_type(&self) -> bool {
        self.is_integral_type() || self.is_floating_point_type()
    }

    /// arithmetic types or pointer types
    pub fn is_scalar_type(&self) -> bool {
        self.is_arithmetic_type() || self.kind.is_ptr()
    }

    pub fn get_record_kind(&self) -> Option<RecordKind> {
        match &self.kind {
            TypeKind::Tag(x) => match x {
                TagType::Record(x) => Some(x.kind),
                TagType::Enum(x) => None,
            },
            _ => None,
        }
    }

    pub fn is_structure_type(&self) -> bool {
        self.get_record_kind()
            .map(|x| x == RecordKind::Struct)
            .unwrap_or(false)
    }

    pub fn is_union_type(&self) -> bool {
        self.get_record_kind()
            .map(|x| x == RecordKind::Union)
            .unwrap_or(false)
    }

    pub fn is_enum_type(&self) -> bool {
        match &self.kind {
            TypeKind::Tag(x) => match x {
                TagType::Record(x) => false,
                TagType::Enum(x) => true,
            },
            _ => false,
        }
    }

    ///  array types or structure types
    pub fn is_aggregate_type(&self) -> bool {
        self.kind.is_array() || self.is_structure_type()
    }
}
