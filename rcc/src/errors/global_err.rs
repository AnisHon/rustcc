use crate::errors::lex_error::LexError;
use crate::errors::parser::ParserError;

#[derive(Debug)]
pub enum GlobalError {
    LexError(LexError),
    ParseError(ParserError),
}
