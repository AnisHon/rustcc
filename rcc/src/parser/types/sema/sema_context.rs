use crate::err::parser_error::ParserResult;

pub enum ContextType {
    Struct,
    Enum,
    Parameter,
    FuncBody
}

pub struct SemaContext {
    
}

impl SemaContext {
    pub fn new() -> Self {
        todo!()
    }

    pub fn enter(&mut self, context: ContextType) -> ParserResult<()>{
        todo!()
    }

    pub fn exit(&mut self) -> ParserResult<()>{
        todo!()
    }
}