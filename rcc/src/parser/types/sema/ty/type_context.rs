use crate::err::parser_error::{ParserError, ParserResult};
use crate::parser::types::decl_spec::{DeclSpec, TypeQual, TypeQualKind, TypeQualType, TypeSpec, TypeSpecKind};
use crate::parser::types::declarator::Declarator;
use crate::parser::types::sema::sema_type::{Qualifier, Type, TypeKind};
use rustc_hash::{FxHashMap, FxHashSet};
use std::rc::Rc;
use crate::parser::types::ast::decl::Decl;
use crate::parser::types::common::Ident;

pub struct TypeContext {
    name: FxHashMap<String, Rc<Type>>,
    types: FxHashSet<Rc<Type>>
}

impl TypeContext {
    pub fn new() -> Self {
        Self {
            name: FxHashMap::default(),
            types: FxHashSet::default(),
        }
    }

    fn get_or_set(&mut self, ty: Rc<Type>) -> Rc<Type> {
        match self.types.get(&ty) {
            Some(x) => Rc::clone(x),
            None => self.types.insert(Rc::clone(&ty))
        }
        ty
    }

    fn resolve_qualifier(&mut self, qual: &TypeQualType) -> Qualifier {
        Qualifier {
            is_const: qual[TypeQualKind::Const as usize].is_some(),
            is_volatile: qual[TypeQualKind::Volatile as usize].is_some(),
            is_restrict: qual[TypeQualKind::Restrict as usize].is_some(),
        }
    }

    fn resolve_type_base(&mut self, qual: &TypeQualType, specs: &[TypeSpec]) -> ParserResult<Rc<Type>> {
        let qualifier = self.resolve_qualifier(qual);
        let mut int = None;
        let mut signed = None;
        let mut state = TypeSpecState::Init;
        let mut decl = None;
        for spec in specs {
            let next_state = match &spec.kind {
                TypeSpecKind::Int => {
                    if int.is_some() {
                        todo!()
                    }
                    int = Some(spec);
                    TypeSpecState::Int
                }
                TypeSpecKind::Signed
                | TypeSpecKind::Unsigned => {
                    if signed.is_some() {
                        todo!()
                    }
                    signed = Some(spec);
                    continue
                }
                TypeSpecKind::Void => TypeSpecState::Void,
                TypeSpecKind::Char => TypeSpecState::Char,
                TypeSpecKind::Short => TypeSpecState::Short,
                TypeSpecKind::Long => TypeSpecState::Long,
                TypeSpecKind::Float => TypeSpecState::Float,
                TypeSpecKind::Double => TypeSpecState::Double,
                TypeSpecKind::Struct(x) => {
                    decl = Some(x);
                    TypeSpecState::Struct
                },
                TypeSpecKind::Union(x) => {
                    decl = Some(x);
                    TypeSpecState::Union
                },
                TypeSpecKind::Enum(x) => {
                    decl = Some(x);
                    TypeSpecState::Enum
                },
                TypeSpecKind::TypeName(_, x) => {
                    decl = Some(x);
                    TypeSpecState::TypeName
                },
            };
            state = match combine(state, next_state) {
                Some(state) => state,
                None => {
                    todo!();
                }
            };
        }

        match state {
            TypeSpecState::Void => {
                Type::new(qualifier, TypeKind::Void)
            },
            TypeSpecState::Char
            | TypeSpecState::Short
            | TypeSpecState::Int
            | TypeSpecState::Long
            | TypeSpecState::LongLong => {
                todo!()
            }
            TypeSpecState::Float
            | TypeSpecState::Double
            | TypeSpecState::LongDouble => {
                todo!()
            }
            TypeSpecState::Struct
            | TypeSpecState::Union
            | TypeSpecState::Enum
            | TypeSpecState::TypeName => decl.unwrap().ty.unwrap(),
            _ => todo!()
        }


        todo!()
    }

    pub fn resolve_decl_spec(&mut self, decl_spec: &DeclSpec) -> ParserResult<Rc<Type>> {
        todo!()
    }

    pub fn resolve_declarator(&mut self, declarator: &Declarator) -> ParserResult<Rc<Type>> {
        // declarator.decl_spec
        todo!()
    }
}


#[derive(Debug, Clone, Copy)]
enum TypeSpecState {
    Init,
    Void,
    Char,
    Short,
    Int,
    Long,
    LongLong,
    Float,
    Double,
    LongDouble,
    Struct,
    Union,
    Enum,
    TypeName,
}

fn combine(state1: TypeSpecState, state2: TypeSpecState) -> Option<TypeSpecState> {
    use TypeSpecState::*;
    match (state1, state2) {
        (Init, _) => Some(state2),
        (Void, _) => None,
        (Char, Int) => Some(Char),
        (Short, Int) => Some(Short),
        (Int, Char) => Some(Char),
        (Int, Short) => Some(Short),
        (Int, Long) => Some(Int),
        (Int, LongLong) => Some(LongLong),
        (Long, Int) => Some(Long),
        (Long, Long) => Some(LongLong),
        (Long, Double) => Some(LongDouble),
        (LongLong, Int) => Some(LongLong),
        (Float, _) => None,
        (Double, Long) => Some(LongDouble),
        (LongDouble, _) => None,
        (Struct, _) => None,
        (Union, _) => None,
        (Enum, _) => None,
        (TypeName, _) => None,
        (_, _) => None,
    }
}

