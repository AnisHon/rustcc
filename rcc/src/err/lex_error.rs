use thiserror::Error;

pub type LexResult<T> = Result<T, LexError>;

#[derive(Error, Debug)]
pub enum LexError {
    #[error("unexpected symbol: {symbol}")]
    InvalidToken { pos: usize, symbol: String },
    #[error("missing terminating: {content} character")]
    MissingTerminating { pos: usize, content: &'static str },
    #[error("unterminated comment")]
    UnterminatedComment { pos: usize }
}



