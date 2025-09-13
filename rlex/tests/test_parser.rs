use rlex::automata_builder::nfa_builder::NFABuilder;
use rlex::char_class::char_class_builder::CharClassBuilder;
use rlex::lex::lex_core::re2tokens;
use rlex::parser::cst::CSTNode;
use rlex::parser::parser_core::ReParser;

#[test]
fn test_re_parser() {
    let tokens = match re2tokens("\"([^\"\\n]|\\.)*\"") {
        Ok(tokens) => tokens,
        Err(e) => panic!("{:}", e),
    };

    // println!("{:#?}", tokens);

    let parser = match ReParser::new(tokens) {
        Ok(tokens) => tokens,
        Err(e) => panic!("{:}", e),
    };

    if let CSTNode::Expr(expr) = parser.get_cst() {
        if let CSTNode::Alternation(alter) = &**expr {
            for x in alter {
                println!("{:?}", x);
            }
        }
    }
    println!("ST  : {:?}", parser.get_cst());
    println!("AST : {:?}", parser.get_ast());

    let builder = CharClassBuilder::new((0, 0x10FFFF));
    let c = builder.build_char_class_set(vec![parser.get_ast()]);
    // println!("{:?}", c.find_interval('a', 'z'));
    // println!("{:?}", c.find_reverse_interval('a', 'z'));

    let mut builder = NFABuilder::new(c);
    let nfa = builder.build(parser.get_ast());
    println!("{:?}", nfa);
}
