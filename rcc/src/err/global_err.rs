use crate::err::lex_error::LexError;
use crate::err::parser_error::ParserError;

#[derive(Debug)]
pub enum GlobalError {
    LexError(LexError),
    ParseError(ParserError),
}


