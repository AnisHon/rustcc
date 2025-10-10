use std::sync::{mpsc, Arc};
use crate::content_manager::ContentManager;
use crate::lex::lex_core::{run_async, Lex};

#[test]
fn test() {
    let content = include_str!("../../resources/test.c");
    let manager = ContentManager::new(content.to_string());
    let mut lex = Lex::new(Arc::new(manager));
    while let Some(x) = lex.next_token().unwrap() {
        println!("{:?}.", x)
    }
}

#[test]
fn test_async() {
    let content = include_str!("../../resources/test.c");
    let manager = ContentManager::new(content.to_string());
    let lex = Lex::new(Arc::new(manager));
    let (token_tx, token_rx) = crossbeam_channel::bounded(100);
    let (error_tx, error_rx) = mpsc::channel();

    run_async(lex, token_tx, error_tx);

    for r in token_rx {
        println!("{:?}.", r);
    }

    for r in error_rx {
        println!("{:?}.", r);
    }



}