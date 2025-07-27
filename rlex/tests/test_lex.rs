use rlex::lex::lex::re2tokens;

#[test]
fn test_re2tokens() {
    let tokens = re2tokens(r"[a-z]\w[a-x{}\.[]]").unwrap();
    for token in tokens.iter() {
        print!("{:?} ", token.typ);
    }
    println!();
    println!("{:?}", tokens);

}