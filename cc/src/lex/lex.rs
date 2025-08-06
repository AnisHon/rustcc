use crate::lex::lex_yy::{find_next, find_token, INIT_STATE};

#[test]
fn test_lex() {
    let mut state = INIT_STATE;
    for chr in r"...".chars() {
        println!("{:?}", find_token(state));
        state = match find_next(state, chr) {
            None => panic!("Wrong"),
            Some(x) => x,
        };
    }

    println!("{:?}", find_token(state));
}
