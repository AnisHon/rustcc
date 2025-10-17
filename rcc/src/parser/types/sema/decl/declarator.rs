use std::rc::Rc;
use crate::err::parser_error::ParserResult;
use crate::parser::types::ast::decl::{Decl, DeclKind};
use crate::parser::types::declarator::Declarator;
use crate::parser::types::sema::Sema;
use crate::parser::types::sema::sema_type::Type;

impl Sema {
    pub fn act_on_declarator(&mut self, declarator: Declarator) -> ParserResult<Rc<Decl>> {
        todo!()
    }
    
    pub fn act_on_var_init(&mut self, kind: &DeclKind) -> ParserResult<Rc<Type>> {
        todo!()
    }
    
    pub fn act_on_enum(&mut self, kind: &DeclKind) -> ParserResult<Rc<Type>> {
        match kind {
            DeclKind::Enum { .. } => {}
            DeclKind::EnumRef { .. } => {}
            _ => unreachable!()
        }
        todo!()
    }
    
    pub fn act_on_field(&mut self, kind: &DeclKind) -> ParserResult<Rc<Type>> {
        let (decl, colon, bit_field) = kind.as_field().unwrap();
        
        todo!()
    }
    
    pub fn act_on_record(&mut self, kind: &DeclKind) -> ParserResult<Rc<Type>> {
        todo!()
    }
    
}