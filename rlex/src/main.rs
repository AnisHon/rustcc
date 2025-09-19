use rlex::rlexer::lex_config::LexConfigParser;
use rlex::rlexer::lex_writer::LexWriter;
use rlex::rlexer::lexer::Lexer;


pub fn get_path(path: &str) -> String {
    format!("{}{}", env!("CARGO_MANIFEST_DIR"), path)
}
fn main() {
    
    let lex_input = include_str!("../../src/clex.l");

    let parser = LexConfigParser::new(lex_input.to_owned());
    let config = parser.parse();
    // println!("{:#?}", config);
    
    let lex = Lexer::new(config);

    let dfa = lex.get_dfa();

    // let char_class_set = lex.get_char_class_set();
    // let mut state = dfa.get_init_state();
    // for chr in "// 123".chars() {
    //     println!("{:?}", state);
    //     let c = char_class_set.find_char(chr);
    //     println!("{:?}", dfa.get_meta(state));
    //     state = match dfa.find_next(state, c) {
    //         None => panic!("Wrong"),
    //         Some(x) => x,
    //     };
    // }
    // println!("{:?}", dfa.get_symbols(state));
    // println!("{:?}", dfa.get_meta(state));
    // 
    // println!("{:?}", dfa.get_meta(state));

    println!("size: {} x {}", dfa.size(), dfa.get_stride());
    let mut sum = 0;
    for x in 0..dfa.size() {
        sum += dfa.get_symbols(x).len()
    }
    println!("edges: {}", sum);

    let lex_writer = LexWriter::new(&get_path("/../src/gen/lex_yy.rs"), lex, );
    lex_writer.write();
}
