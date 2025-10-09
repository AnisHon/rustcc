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
    #[error("Invalid suffix '{content} on on {typ} constant")]
    InvalidSuffix { beg: usize, end: usize, content: String, typ: &'static str },

}



