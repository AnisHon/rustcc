use crate::types::span::Span;

#[derive(Debug)]
pub enum DeclWarning {
    TypedefNoName(Span), // typedef requires a name
}
