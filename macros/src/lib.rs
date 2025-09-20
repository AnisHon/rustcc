//!
//! proc_macro2是社区提供的兼容层，然后就喝官方一个名字
//! 我很反感混乱的名称空间，所以我坚持加上名称空间不会直接导入 proc_macro2 或者 proc_macro，
//!

use std::cmp::Ordering;
use quote::quote;
use syn::{DeriveInput, Fields, Variant};


fn is_ignore(variant: &Variant) -> bool {
    for attr in &variant.attrs {
        if attr.path().is_ident("enum_auto_ignore") {
            return true
        }
    }
    false
}

///
/// EnumAutoInto，实际实现的是From
///
fn impl_into(ast: &DeriveInput) -> proc_macro2::TokenStream {
    if !ast.generics.params.is_empty() {
        panic!("generic is not supported");
    }
    let name = &ast.ident;
    let enum_data = if let syn::Data::Enum(data) = &ast.data {
        data
    } else {
        panic!("{} is not an enum", name);
    };

    let mut impls: proc_macro2::TokenStream = proc_macro2::TokenStream::new();

    for variant in &enum_data.variants {
        if is_ignore(variant) {
            continue;
        }

        let variant_name = &variant.ident;

        let (target_type, into_body) = match &variant.fields {
            Fields::Unnamed(unnamed) => {

                let field_types: Vec<_> = unnamed.unnamed.iter().map(|f| &f.ty).collect();
                let field_sz = field_types.len();

                let field_ty = match field_sz.cmp(&1) {
                    Ordering::Equal => *field_types.first().unwrap(),
                    _ => panic!("{} expect 1 fields, got {}", variant_name, field_sz),
                };

                let target_type = if field_sz == 1 {
                    quote!(#field_ty)
                } else {
                    panic!("{} expect 1 fields, got {}", variant_name, field_sz);
                };

                let into_body = quote!(
                    match self {
                        #name::#variant_name(x) => x,
                        _ => panic!("Expected {}::{}, found {:?}", stringify!(#name), stringify!(#variant_name), self)
                    }
                );
                (target_type, into_body)
            },
            _ => panic!("{variant_name} is not unnamed(tuple) Variant!")
        };

        let impl_block = quote!(
            impl Into<#target_type> for #name {
                fn into(self) -> #target_type {
                    #into_body
                }
            }
        );

        impls.extend(impl_block);
    }

    impls
}

fn get_type(variant: &Variant) -> proc_macro2::TokenStream {
    let variant_name = &variant.ident;
    match &variant.fields {
        Fields::Unnamed(unnamed) => {

            let field_types: Vec<_> = unnamed.unnamed.iter().map(|f| &f.ty).collect();
            let field_sz = field_types.len();

            let field_ty = match field_sz.cmp(&1) {
                Ordering::Equal => *field_types.first().unwrap(),
                _ => panic!("{} expect 1 fields, got {}", variant_name, field_sz),
            };

            let recv_type = if field_sz == 1 {
                quote!(#field_ty)
            } else {
                panic!("{} expect 1 fields, got {}", variant_name, field_sz);
            };

            recv_type
        },
        _ => panic!("{variant_name} is not unnamed(tuple) Variant!")
    }
}

fn impl_from(ast: &DeriveInput) -> proc_macro2::TokenStream {
    if !ast.generics.params.is_empty() {
        panic!("generic is not supported");
    }
    let name = &ast.ident;
    let enum_data = if let syn::Data::Enum(data) = &ast.data {
        data
    } else {
        panic!("{} is not an enum", name);
    };

    let mut impls: proc_macro2::TokenStream = proc_macro2::TokenStream::new();

    for variant in &enum_data.variants {
        if is_ignore(variant) { // 忽略ignore
            continue;
        }
        let variant_name = &variant.ident;

        let recv_type = get_type(variant);

        let impl_block = quote!(
            impl From<#recv_type> for #name {
                fn from(value: #recv_type) -> Self {
                    Self::#variant_name(value)
                }
            }
        );

        impls.extend(impl_block);
    }

    impls
}

///
/// EnumAutoInto，实际实现的是From
///
fn impl_into_option(ast: &DeriveInput) -> proc_macro2::TokenStream {
    if !ast.generics.params.is_empty() {
        panic!("generic is not supported");
    }
    let name = &ast.ident;
    let enum_data = if let syn::Data::Enum(data) = &ast.data {
        data
    } else {
        panic!("{} is not an enum", name);
    };

    let mut impls: proc_macro2::TokenStream = proc_macro2::TokenStream::new();

    for variant in &enum_data.variants {
        if is_ignore(variant) {
            continue;
        }
        
        let target_type = get_type(variant);

        let impl_block = quote!(
            impl Into<Option<#target_type>> for #name {
                fn into(self) -> Option<#target_type> {
                    match self {
                        #name::None => None,
                        _ => Some(self.into())

                    }
                }
            }
        );

        impls.extend(impl_block);
    }

    impls
}


/// 通过`From<U> For T` 使支持`Into<T> For U`，更方便的拆箱, U是枚举本身，T是枚举内部类型
///
/// - 追求绝对简单
/// - 不支持范型
/// - 只支持单个元组的元组枚举（或者说匿名字段）
/// - 失败直接panic
#[proc_macro_derive(EnumAutoInto, attributes(enum_auto_ignore))]
pub fn derive_into(input: proc_macro::TokenStream) -> proc_macro::TokenStream {

    let ast: DeriveInput = syn::parse(input).unwrap();

    let stream = impl_into(&ast);

    proc_macro::TokenStream::from(stream)
}

/// 自动实现From<T>，方便的装箱
/// - 追求绝对简单
/// - 不支持范型
/// - 只支持单个元组的元组枚举（或者说匿名字段）
#[proc_macro_derive(EnumAutoFrom, attributes(enum_auto_ignore))]
pub fn derive_from(input: proc_macro::TokenStream) -> proc_macro::TokenStream {

    let ast: DeriveInput = syn::parse(input).unwrap();

    let stream = impl_from(&ast);

    proc_macro::TokenStream::from(stream)
}


/// 自动实现Into<Option<T>>，方便拆箱后装箱成Option<T>
/// - 追求绝对简单
/// - Option::None 转换成 T::None，必须有None字段
/// - 不支持范型
/// - 只支持单个元组的元组枚举（或者说匿名字段）
#[proc_macro_derive(EnumAutoIntoOption, attributes(enum_auto_ignore))]
pub fn derive_into_option(input: proc_macro::TokenStream) -> proc_macro::TokenStream {

    let ast: DeriveInput = syn::parse(input).unwrap();

    let stream = impl_into_option(&ast);

    proc_macro::TokenStream::from(stream)
}