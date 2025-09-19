use thiserror::Error;

pub type LexResult<T> = Result<T, LexError>;

#[derive(Error, Debug)]
pub enum LexError {
    #[error("Unexpected Character: {chr}")]
    UnexpectedCharacter{ pos: usize, chr: char },
    
}



