use std::error::Error;

mod container;
pub mod lex;
pub mod utils;

pub type ErrResult<T> = Result<T, Box<dyn Error>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
