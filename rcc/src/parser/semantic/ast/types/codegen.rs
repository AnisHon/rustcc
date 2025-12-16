use crate::parser::ast::types::{Qualifier, Type, TypeKind};


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

impl Type {
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
                code.push_str(&elem_ty.upgrade().unwrap().to_code());
                code.push(' ');
            }
            TypeKind::Array{ size, elem_ty } => {
                code.push_str(&elem_ty.upgrade().unwrap().to_code());
                code.push_str(&size.to_code());
                code.push(' ');
            }
            TypeKind::Function{ ret_ty, params, is_variadic } => {
                code.push_str("fn ");
                let param = params.iter()
                    .map(|x| x.upgrade().unwrap().to_code())
                    .collect::<Vec<_>>()
                    .join(",");
                let variadic = is_variadic.then(|| ",...").unwrap_or_default();
                code.push_str(&format!("({}{})", param, variadic));
                code.push_str(" -> ");
                code.push_str(&ret_ty.upgrade().unwrap().to_code());
                code.push(' ');
            }
            TypeKind::Struct{ name, fields, .. } => {
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
            TypeKind::Union{ name, fields, .. } => {
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