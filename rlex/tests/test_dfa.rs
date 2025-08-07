use rlex::automata_builder::dfa_builder::DFABuilder;
use rlex::automata_builder::dfa_optimizer::DFAOptimizer;
use rlex::automata_builder::nfa_builder::NFABuilder;
use rlex::char_class::char_class_builder::CharClassBuilder;
use rlex::lex::lex::re2tokens;
use rlex::parser::parser::ReParser;

#[test]
fn test_dfa_builder() {
    let tokens = match re2tokens(r"'([^\\'\n]|\\.)*'") {
        Ok(tokens) => tokens,
        Err(e) => panic!("{:}", e),
    };

    let parser = match ReParser::new(tokens) {
        Ok(tokens) => tokens,
        Err(e) => panic!("{:}", e),
    };

    let builder = CharClassBuilder::new((0, 0x10FFFF));
    let char_class_set = builder.build_char_class_set(vec![parser.get_ast()]);

    let mut builder = NFABuilder::new(char_class_set.clone());
    let nfa = builder.build(parser.get_ast());

    let dfa_builder = DFABuilder::new(nfa, char_class_set.size(), |x, y| {
        let mut prev = 0;
        for &i in y {
            let states = x.get_status(i);
            prev = i;
            if states.terminate {
                break;
            }
        }
        prev
    });
    let dfa = dfa_builder.build();
    let optimizer = DFAOptimizer::new(
        dfa,
        |meta| meta.terminate as usize,
        |dfa, states| {
            let mut x = 0;
            for (state_id, meta) in states.iter().map(|&x| (x, dfa.get_meta(x))) {
                x = state_id;
                if meta.terminate {
                    break;
                }
            }
            x
        },
    );

    let dfa = optimizer.optimize();

    let mut state = dfa.get_init_state();

    for chr in r"'\n'".to_string().chars() {
        println!("{:?}", dfa.get_meta(state));
        let chr = char_class_set.find_char(chr);
        state = match dfa.find_next(state, chr) {
            Some(state) => state,
            None => panic!("Wrong!"),
        };
    }
    println!("{:?}", dfa.get_meta(state));
}
