use crate::parser::ast::ast_nodes::*;
use crate::parser::ast::parser_node::ParserNode;
use crate::parser::ast::temp_nodes::DeclaratorChunk;
use crate::parser::span::Span;
use crate::types::token::Token;

impl DeclaratorChunk {
    // pub fn make_pointer(ptr: Token) -> ParserNode {
    //     let span = Span::from_token(&ptr);
    //     DeclaratorChunk::Pointer(span).into()
    // }
    //
    // pub fn make_array(lbracket: Token, constexpr: Expression, rbracket: Token) -> ParserNode {
    //     let span = Span::from_tokens(&[lbracket, rbracket]);
    //     DeclaratorChunk::Array(constexpr, span).into()
    // }

}
