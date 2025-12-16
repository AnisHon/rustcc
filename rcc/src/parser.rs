use std::sync::{mpsc, Arc};
use crate::content_manager::ContentManager;
use crate::lex::lex_core::{run_lexer, Lex};
use crate::lex::types::token_kind::TokenKind;
use crate::parser::parser_core::Parser;

mod parser_decl;
mod parser_expr;
pub mod parser_core;
mod semantic;
mod parser_stmt;
mod parser_function;

pub use crate::parser::semantic::{ast, sema::Sema, common};

#[cfg(test)]
mod test {
    use std::rc::Rc;
    use crate::lex::types::token_kind::Symbol;
    use crate::parser::common::Ident;
    use crate::parser::Sema;
    use crate::parser::semantic::decl_spec::{DeclSpec, ParamDecl, ParamList, TypeQualType, TypeSpec, TypeSpecKind};
    use crate::parser::semantic::declarator::{Declarator, DeclaratorChunk, DeclaratorChunkKind};
    use crate::types::span::{Pos, Span};

    #[test]
    pub fn test_() {
        let mut sema = Sema::new();


        let mut declarator = Declarator::new(Rc::from(DeclSpec { storage: None, type_specs: vec![TypeSpec { kind: TypeSpecKind::Int, span: Span::default() }], type_quals: TypeQualType::default(), func_spec: None, span: Span::default() }));
        declarator.name = Option::from(Ident { symbol: Symbol::new("123"), span: Span::default() });
        declarator.chunks.push(DeclaratorChunk::new(DeclaratorChunkKind::Function { l: Pos::default(), r: Pos::default(), param: ParamDecl::Params(ParamList::default()) }, Default::default()));

        let cp = declarator.clone();


        let new = sema.act_on_declarator(cp);
        let old = sema.act_on_declarator(declarator);
        // println!("{:#?}", old);
        // println!("{:#?}", new);

        let old = old.unwrap().ty;
        let new = new.unwrap().ty;

        println!("{:?}", old);
        println!("{:?}", new);

        println!("{:#?}", new == old);
        println!("{:?}", old.kind.as_function().unwrap().1.as_ptr() == new.kind.as_function().unwrap().1.as_ptr());
        // println!("{:?}", Rc::as_ptr(&new));
    }

}