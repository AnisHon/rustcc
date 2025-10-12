use crate::err::parser_error::ParserResult;
use crate::parser::parser_core::Parser;

impl Parser {
    
    pub(crate) fn parse_type_name(&mut self) -> ParserResult<()> {
        // todo
        self.stream.next();
        Ok(())
    }

    pub(crate) fn parse_decl(&mut self) -> ParserResult<()> {
        todo!()
    }
    
}