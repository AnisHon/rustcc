use std::error::Error;
use std::fmt::{Display};


pub type ReResult<T> = Result<T, ReError>;

#[derive(Debug)]
pub struct ReError {
    msg: String,
    pos: usize,
    re: String,
}

impl ReError {
    pub fn new(msg: String, pos: usize, re: String) -> ReError {
        ReError { msg, pos, re }
    }
}

impl Display for ReError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error in Regex[{}]: {} \nMessage: {}", self.re, self.pos, self.msg)
    }
}

impl Error for ReError {
}


