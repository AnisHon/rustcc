pub mod common;
pub mod decl_error;
pub mod decl_warning;
pub mod scope_error;
pub mod type_error;

pub use common::ParserError;
pub type ParserResult<T> = Result<T, ParserError>;
