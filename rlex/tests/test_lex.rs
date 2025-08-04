use rlex::lex::lex::re2tokens;

#[test]
fn test_re2tokens() {
    let tokens = match re2tokens(r"[a-z]{1,2 }\w[a-x{}\.[]]") {
        Ok(tokens) => tokens,
        Err(e) => panic!("{:}", e),
    };


    for token in tokens.iter() {
        print!("{:?} ", token.typ);
    }
    println!();
    println!("{:?}", tokens);

}
