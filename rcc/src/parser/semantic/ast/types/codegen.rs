use crate::parser::{
    ast::types::{Qualifier, Type},
    semantic::comp_ctx::CompCtx,
};

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
    pub fn to_code(&self, ctx: &CompCtx) -> String {
        // 抽到外面
        let mut code = String::new();
        let qual = self.qual.to_code();

        code.push_str(&qual);

        todo!()
        // match &self.kind {
        //     TypeKind::Void => code.push_str("void "),
        //     TypeKind::Integer{ is_signed, size } => {
        //         if *is_signed {
        //             code.push_str("signed ");
        //         } else {
        //             code.push_str("unsigned ");
        //         }
        //         code.push_str(size.to_code());
        //         code.push(' ');
        //     },
        //     TypeKind::Floating{ size } => {
        //         code.push_str(size.to_code());
        //         code.push(' ');
        //     }
        //     TypeKind::Pointer{ elem_ty } => {
        //         code.push('*');
        //         let elem_code = ctx.type_ctx.get_type(*elem_ty).to_code(ctx);
        //         code.push_str(elem_code.as_str());
        //         code.push(' ');
        //     }
        //     TypeKind::Array{ size, elem_ty } => {
        //         let elem_code = ctx.type_ctx.get_type(*elem_ty).to_code(ctx);
        //         code.push_str(elem_code.as_str());
        //         code.push_str(&size.to_code());
        //         code.push(' ');
        //     }
        //     TypeKind::Function{ ret_ty, params, is_variadic } => {
        //         code.push_str("fn ");
        //         let param = params.iter()
        //             .map(|x| ctx.type_ctx.get_type(*x).to_code(ctx))
        //             .collect::<Vec<_>>()
        //             .join(",");
        //         let variadic = is_variadic.then(|| ",...").unwrap_or_default();
        //         code.push_str(&format!("({}{})", param, variadic));
        //         code.push_str(" -> ");
        //         let ret_code = ctx.get_type(*ret_ty).to_code(ctx);
        //         code.push_str(ret_code.as_str());
        //         code.push(' ');
        //     }
        //     TypeKind::Record{ name, fields, .. } => {
        //         let name = name.as_ref().map(|x| x.symbol.get()).unwrap_or_default();
        //         let fields: String = fields.iter().map(|x| x.to_code(ctx)).collect();
        //         code.push_str("struct ");
        //         code.push_str(name);
        //         code.push('{');
        //         code.push_str(&fields);
        //         code.push('}');
        //         code.push(' ');
        //     }
        //     TypeKind::Enum{ name, fields, .. } => {
        //         let name = name.as_ref().map(|x| x.symbol.get()).unwrap_or_default();
        //         let fields = fields.iter()
        //             .map(|x| x.to_code())
        //             .collect::<Vec<_>>()
        //             .join(",");
        //         code.push_str("union ");
        //         code.push_str(name);
        //         code.push('{');
        //         code.push_str(&fields);
        //         code.push('}');
        //         code.push(' ');
        //     }
        //     TypeKind::Unknown => {
        //         code.push_str("$ERROR$");
        //     }
        // }
    }
}
