use std::fmt::format;
use crate::err::parser_error::{ParserError, ParserResult, PartialResult};
use crate::types::ast::decl_info::{ArraySizeModifier, DeclChunk, DeclChunkKind, DeclSpec, Declarator, PointerChunk, TypeQual, TypeSpec};
use crate::types::ast::func_info::ParamList;
use crate::types::ast::nodes::{ConstantKind, IntegerSize, Qualifiers, StorageClass, Type, TypeKind};

pub(crate) enum TypeSuffix {
    Pointer(PointerChunk),
    DeclChunk(DeclChunk),
}

pub(crate) fn build_type_decl(decl_spec: DeclSpec, declarator: Option<Declarator>) {
    let result = list2qualifiers(decl_spec.type_quals);



}


///
/// 将有递归结构的declarator按照语义优先级展平，直接遍历
///
pub(crate) fn flat_chunks(declarator: Declarator, out: &mut Vec<TypeSuffix>) -> Option<String> {
    // 压入指针
    out.extend(
        declarator.pointer_chunks.into_iter()
            .map(|x| TypeSuffix::Pointer(x))
    );

    // DeclChunk的解析顺序和实际顺序相反
    for x in declarator.decl_chunks.into_iter().rev() {
        match x.chunk {
            DeclChunkKind::Ident { name } => return Some(name),
            DeclChunkKind::Paren(x) => return flat_chunks(x.inner, out),
            _ => out.push(TypeSuffix::DeclChunk(x)),
        }
    }

    None
}

/// 将展平的chunk转换成树状类型
pub(crate) fn chunks2type(chunks: Vec<TypeSuffix>, base: Box<Type>) -> PartialResult<Box<Type>> {
    let mut errors: Vec<ParserError> = Vec::new();

    let mut curr = base;
    for x in chunks {
        match x {
            TypeSuffix::Pointer(x) => {
                let mut result = list2qualifiers(x.quals);
                errors.append(&mut result.errors);

                let kind = TypeKind::Pointer(curr);
                curr = Box::new(Type::new(kind, x.span));
            }
            TypeSuffix::DeclChunk(x) => {
                match x.chunk {
                    DeclChunkKind::Array { size, asm } => {
                        let kind = match asm {
                            ArraySizeModifier::Normal => {
                                TypeKind::Array { elem_ty: curr, size: None }
                            }
                            ArraySizeModifier::Static => {
                                let size = size.unwrap().get_constant().unwrap();
                                let size = match size.kind {
                                    ConstantKind::Int(x) => x,
                                    _ => panic!("size of array has non-integer type")
                                };
                               TypeKind::Array { elem_ty: curr, size: Some(size) } }
                            ArraySizeModifier::VLA => unreachable!() // 未实现
                        };
                        curr = Box::new(Type::new(kind, x.span))
                    }
                    DeclChunkKind::Function { param_list } => {
                        let kind = match param_list.inner {
                            None => TypeKind::Function { ret_ty: curr, params: vec![], is_variadic: false },
                            Some(x) => {
                                let types = x.list.list.into_iter().map(|x| x.ty).collect();
                                TypeKind::Function { ret_ty: curr, params: types, is_variadic: x.is_variadic }
                            }
                        };
                        curr = Box::new(Type::new(kind, x.span));
                    }
                    DeclChunkKind::KRFunction { param_list } => {
                        todo!() // KR声明

                    }
                    _ => unreachable!()
                }
            }
        }
    }

    PartialResult::new(curr, errors)
}

pub(crate) fn list2qualifiers(quals: Vec<TypeQual>) -> PartialResult<Qualifiers> {
    let mut result: PartialResult<Qualifiers> = PartialResult::default();
    for x in quals {
        match x {
            TypeQual::Const(x) => {
                if result.data.is_const {
                    ParserError::warning(x, "Duplicate 'const' declaration specifier".to_owned());
                }
                result.data.is_const = true;
            },
            TypeQual::Volatile(x) => {
                if result.data.is_volatile {
                    ParserError::warning(x, "Duplicate 'volatile' declaration specifier".to_owned());
                }
                result.data.is_volatile = true;
            }
        }
    }
    result
}

pub(crate) fn list2storage(storage: Vec<StorageClass>) -> PartialResult<Option<StorageClass>> {
    let errors = Vec::new();

    if storage.len() > 1 {
        let msg = format!("Cannot combine with previous '{}' declaration specifier", storage[0].to_string());
    }
    
    PartialResult::new(storage.get(0).cloned(), errors)
}

// pub(crate) fn list2spec(specs: Vec<TypeSpec>) -> PartialResult<Type> {
//     // 目前不支持long long long int之类的
//     let kind =
//     match specs[0] {
//         TypeSpec::Void(_) => TypeSpec::Void
//         TypeSpec::Char(_) => {}
//         TypeSpec::Short(_) => {}
//         TypeSpec::Int(_) => {}
//         TypeSpec::Long(_) => {}
//         TypeSpec::Signed(_) => {}
//         TypeSpec::Unsigned(_) => {}
//         TypeSpec::Float(_) => {}
//         TypeSpec::Double(_) => {}
//         TypeSpec::StructOrUnion(_) => {}
//         TypeSpec::Enum(_) => {}
//         TypeSpec::TypeName(_, _) => {}
//     }
//
// }