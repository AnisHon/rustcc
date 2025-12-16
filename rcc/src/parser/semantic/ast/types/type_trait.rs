use std::hash::{Hash, Hasher};
use crate::parser::ast::types::{Qualifier, Type, TypeKind};
use crate::parser::ast::types::TypeKind::{Array, Enum, EnumRef, Floating, Function, Integer, Pointer, Struct, StructRef, Union, UnionRef, Unknown, Void};

impl Default for Type {
    fn default() -> Self {
        Self {
            qual: Qualifier::default(),
            kind: TypeKind::Unknown,
        }
    }
}

impl Hash for TypeKind {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use crate::parser::semantic::ast::types::type_struct::TypeKind::*;
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
                elem_ty.hash(state);
            }
            Array { elem_ty, size } => {
                4u8.hash(state);
                elem_ty.hash(state);
                size.hash(state);
            }
            Function { ret_ty, params, is_variadic, .. } => {
                5u8.hash(state);
                ret_ty.hash(state);
                params.iter().for_each(|x| x.hash(state));
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
            Struct { name, fields, size } => {
                6u8.hash(state);
                name.hash(state);
                fields.hash(state);
                size.hash(state);
            },
            Union { name, fields, size } => {
                8u8.hash(state);
                name.hash(state);
                fields.hash(state);
                size.hash(state);
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
        use crate::parser::semantic::ast::types::type_struct::TypeKind::*;
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
            ) => ty1 == ty2,
            (
                Array { elem_ty: ty1, size: size1 },
                Array { elem_ty: ty2, size: size2 }
            ) => ty1 == ty2 && size1 == size2,
            (
                Function { ret_ty: ty1, params: params1, is_variadic: variadic1, .. },
                Function { ret_ty: ty2, params: params2, is_variadic: variadic2, .. }
            ) => ty1 == ty2
                && params1.iter().map(|x| x).eq(params2.iter().map(|x| x))
                && variadic1 == variadic2,
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
                Struct { name: name1, fields: fields1, size: sz1 },
                Struct { name: name2, fields: fields2, size: sz2 },
            )
            | (
                Union { name: name1, fields: fields1, size: sz1 },
                Union { name: name2, fields: fields2, size: sz2 }
            ) => sz1 == sz2 && name1 == name2 && fields1 == fields2,
            (
                Enum { name: name1, fields: fields1 },
                Enum { name: name2, fields: fields2 }
            ) => name1 == name2 && fields1 == fields2,
            (_, _) => false,
        }
    }
}

impl Eq for TypeKind {}