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
    pub fn new(msg: &str, pos: usize) -> ReError {
        ReError { msg: msg.to_string(), pos, re: String::new()}
    }

    pub fn with_re(mut self, re: &str) -> Self{
        self.re = re.to_string();
        self
    }

}

impl Display for ReError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error in Regex[{}]: {} \nMessage: {}", self.pos, self.re, self.msg)
    }
}

impl Error for ReError {
}


