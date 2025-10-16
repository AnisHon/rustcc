use std::cell::RefCell;
use std::rc::Rc;
use crate::err::parser_error::ParserResult;
use crate::parser::types::sema::decl_context::DeclContext;

pub enum DeclContextType {
    Struct,
    Enum,
    Parameter,
    FuncBody
}

pub struct SemaContext {
    curr_decl: Rc<RefCell<dyn DeclContext>>
}

impl SemaContext {
    pub fn new() -> Self {
        todo!()
    }

    pub fn enter(&mut self, context: DeclContextType) -> ParserResult<()>{
        todo!()
    }

    pub fn exit(&mut self) -> ParserResult<()>{
        todo!()
    }
}

