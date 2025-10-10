use crate::lex::types::token::Token;
use crate::lex::types::token_kind::{LiteralKind, TokenKind};
use crate::parser::parser_core::Parser;

impl Parser {
    
    fn peek_is_constant(&mut self) -> bool {
        match self.stream.peek() {
            None => false,
            Some(x) => match &x.kind {
                TokenKind::Literal(x) => match x {
                    LiteralKind::Integer { .. }
                    | LiteralKind::Float { .. }
                    | LiteralKind::Char { .. } => true,
                    LiteralKind::String { .. } => false,
                }
                _ => false,
            }
        }
    }
    
    fn peek_is_string(&mut self) -> bool {
        match self.stream.peek() {
            None => false,
            Some(x) => match &x.kind {
                TokenKind::Literal(x) => match x {
                    LiteralKind::Integer { .. }
                    | LiteralKind::Float { .. }
                    | LiteralKind::Char { .. } => false,
                    LiteralKind::String { .. } => true,
                }
                _ => false,
            }
        }
    }
    
    fn parse_string_literal(&mut self) -> Token {
        assert!(self.peek_is_string());
        let token = self.stream.peek();
        while self.peek_is_string() {
            
        }
        
        
        
        todo!()
    }
    
    fn parse_expression() {
        
    }
    
    
    
}