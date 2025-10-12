use std::sync::{mpsc, Arc};
use crate::content_manager::ContentManager;
use crate::lex::lex_core::{run_lexer, Lex};

#[test]
fn test() {
    let content = include_str!("../../resources/example/test.c");
    let manager = ContentManager::new(content.to_string());
    let mut lex = Lex::new(Arc::new(manager));
    while let Some(x) = lex.next_token().unwrap() {
        println!("{:?}.", x)
    }
}

#[test]
fn test_run_lexer() {
    let content = include_str!("../../resources/example/test.c");
    let manager = ContentManager::new(content.to_string());
    let lex = Lex::new(Arc::new(manager));
    let (error_tx, error_rx) = mpsc::channel();

    let vec = run_lexer(lex, error_tx);
    for x in vec {
        println!("{:?}", x);
    }

    for r in error_rx {
        println!("{:?}.", r);
    }



}