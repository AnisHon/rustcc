use std::sync::{mpsc, Arc};
use crate::content_manager::ContentManager;
use crate::lex::lex_core::{run_lexer, Lex};
use crate::lex::types::token_kind::TokenKind;
use crate::parser::parser_core::Parser;

mod parser_decl;
mod parser_expr;
pub mod parser_core;
mod types;
mod parser_stmt;


#[cfg(test)]
mod test {
    use std::sync::{mpsc, Arc};
    use crate::content_manager::ContentManager;
    use crate::lex::lex_core::{run_lexer, Lex};
    use crate::lex::types::token_kind::TokenKind;
    use crate::parser::parser_core::Parser;

    #[test]
    fn test_expr() {
        let code = include_str!("../resources/example/expression.c");
        let content_manager = Arc::new(ContentManager::new(code.to_string()));
        let (error_tx, error_rx) = mpsc::channel();

        // 执行lexer
        let lex = Lex::new(Arc::clone(&content_manager));
        let tokens = run_lexer(lex, error_tx.clone());

        // for x in &tokens {
        //     println!("{:?}", x);
        // }

        let mut parser = Parser::new(tokens, error_tx);

        println!("{:?}", parser.parse_expr().unwrap());
        while let Some(_) = parser.consume(TokenKind::Semi) {
            if parser.check(TokenKind::Eof) {
                break
            }
            println!("{:?}", parser.parse_expr().unwrap());
        }
    }

    #[test]
    fn test_stmt() {
        let code = include_str!("../resources/example/statement.c");
        let content_manager = Arc::new(ContentManager::new(code.to_string()));
        let (error_tx, error_rx) = mpsc::channel();

        // 执行lexer
        let lex = Lex::new(Arc::clone(&content_manager));
        let tokens = run_lexer(lex, error_tx.clone());

        // for x in &tokens {
        //     println!("{:?}", x);
        // }

        let mut parser = Parser::new(tokens, error_tx);

        println!("{:#?}", parser.parse_stmt(false).unwrap())
    }

    #[test]
    fn test_decl() {
        let code = include_str!("../resources/example/declaration.c");
        let content_manager = Arc::new(ContentManager::new(code.to_string()));
        let (error_tx, error_rx) = mpsc::channel();

        // 执行lexer
        let lex = Lex::new(Arc::clone(&content_manager));
        let tokens = run_lexer(lex, error_tx.clone());

        // for x in &tokens {
        //     println!("{:?}", x);
        // }

        let mut parser = Parser::new(tokens, error_tx);

        println!("{:#?}", parser.parse_decl().unwrap())
    }
}


