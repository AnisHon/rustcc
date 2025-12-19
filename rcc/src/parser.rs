pub mod parser_core;
mod parser_decl;
mod parser_expr;
mod parser_function;
mod parser_stmt;
mod semantic;

pub use crate::parser::semantic::{ast, common, comp_ctx, sema::Sema};

#[cfg(test)]
mod test {
    use crate::lex::types::token_kind::Symbol;
    use crate::parser::Sema;
    use crate::parser::common::Ident;
    use crate::parser::semantic::decl_spec::{
        DeclSpec, ParamDecl, ParamList, TypeQuals, TypeSpec, TypeSpecKind,
    };
    use crate::parser::semantic::declarator::{Declarator, DeclaratorChunk, DeclaratorChunkKind};
    use crate::types::span::{Pos, Span};
    use std::rc::Rc;

    #[test]
    pub fn test_() {
        let mut sema = Sema::new();

        let mut declarator = Declarator::new(Rc::from(DeclSpec {
            storage: None,
            type_specs: vec![TypeSpec {
                kind: TypeSpecKind::Int,
                span: Span::default(),
            }],
            type_quals: TypeQuals::default(),
            func_spec: None,
            span: Span::default(),
        }));
        declarator.name = Option::from(Ident {
            symbol: Symbol::new("123"),
            span: Span::default(),
        });
        declarator.chunks.push(DeclaratorChunk::new(
            DeclaratorChunkKind::Function {
                l: Pos::default(),
                r: Pos::default(),
                param: ParamDecl::Params(ParamList::default()),
            },
            Default::default(),
        ));

        let cp = declarator.clone();

        let new = sema.act_on_declarator(cp);
        let old = sema.act_on_declarator(declarator);
        // println!("{:#?}", old);
        // println!("{:#?}", new);

        let old = old.unwrap().ty_key;
        let new = new.unwrap().ty_key;

        println!("{:?}", old);
        println!("{:?}", new);

        println!("{:#?}", new == old);
        println!(
            "{:?}",
            old.kind.as_function().unwrap().1.as_ptr()
                == new.kind.as_function().unwrap().1.as_ptr()
        );
        // println!("{:?}", Rc::as_ptr(&new));
    }
}
