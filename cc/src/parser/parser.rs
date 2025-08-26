use crate::lex::lex_yy::TokenType;
use crate::parser::parser_yy::{get_action, get_goto, LRAction, END_SYMBOL, EXPR_LENS, EXPR_NAMES, INIT_STATE};

#[test]
fn test() {
    // int main();
    let mut x: Vec<_> = [TokenType::KeywordInt, TokenType::Id, TokenType::Lparen, TokenType::Rparen, TokenType::Semicolon].into_iter().map(|x| x as usize).collect();
    x.push(END_SYMBOL);
    println!("{:?}", x);

    let mut state_stack = vec![INIT_STATE];

    let mut iter = x.into_iter().peekable();
    while let Some(&token) = iter.peek() {
        let state = *state_stack.last().unwrap();
        let action = get_action(state, token);
        match action {
            LRAction::Reduce(x) |  LRAction::Accept(x) => {
                println!("reduce {} {}", EXPR_NAMES[x], EXPR_LENS[x]);
                state_stack.drain(state_stack.len() - EXPR_LENS[x]..);

                let state = *state_stack.last().unwrap();
                state_stack.push(get_goto(state, x).unwrap());
                
                if matches!(action, LRAction::Accept(_)) {
                    println!("Accepted");
                    iter.next();;
                }
            }
            LRAction::Shift(x) => {
                state_stack.push(x);
                iter.next();
            }
            LRAction::Error => {
                println!("ERROR");
                iter.next();
            }
        }
    }

}
