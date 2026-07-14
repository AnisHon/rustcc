use thiserror::Error;

pub type LexResult<T> = Result<T, LexError>;

#[derive(Error, Debug)]
pub enum LexError {
    #[error("unexpected symbol: {symbol}")]
    UnknownSymbol{ pos: usize, symbol: char },
    #[error("missing terminating: {chr} character")]
    MissingTerminating { pos: usize, chr: char },
    #[error("unterminated comment")]
    UnterminatedComment { pos: usize },
    #[error("Invalid {invalid} '{content}' on {typ} constant")]
    Invalid { beg: usize, end: usize, invalid: &'static str, content: String, typ: &'static str },
    #[error("Exponent has no digits constant")]
    Exponent { pos: usize },
}



