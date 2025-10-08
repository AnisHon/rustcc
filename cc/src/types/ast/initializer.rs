use crate::types::ast::nodes;
use crate::types::ast::nodes::Initializer;
use crate::types::ast::decl_info::Declarator;
use crate::types::ast::sematic_value::SemanticValue;
use crate::types::lex::token::Token;
use crate::types::span::{SepList, Span};

pub type InitDeclList = SepList<Box<InitDeclarator>>;

#[derive(Debug, Clone)]
pub struct InitDeclarator {
    pub decl: Declarator,
    pub eq_span: Option<Span>,
    pub init: Option<Initializer>,
}

impl InitDeclarator {
    
    pub fn make(decl: Declarator, eq: Option<Token>, init: Option<Initializer>) -> SemanticValue {
        Box::new(Self {
            decl,
            eq_span: eq.map(|x| x.span),
            init,
        }).into()
    }
    
    pub fn make_list(init: Box<InitDeclarator>) -> SemanticValue {
        InitDeclList::new(init).into()
    }
    
    pub fn push(mut list: InitDeclList, comma: Token, init: Box<InitDeclarator>) -> SemanticValue {
        list.push(comma.span, init);
        list.into()
    }
    

}