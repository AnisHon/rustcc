use rlex::rlexer::lex_reader::LexReader;
use rlex::rlexer::lexer::Lexer;

fn get_path(path: &str) -> String {
    format!("{}{}", env!("CARGO_MANIFEST_DIR"), path)
}

fn main() {



    let lex = LexReader::new(&get_path("/resources/clex.l"))
        .read_from_file()
        .unwrap();
    // lex.iter().for_each(|x| println!("{:?}", x));

    println!("{}", lex.len());

    let lex = Lexer::new(lex);

    // let char_class_set = lex.get_char_class_set();
    let dfa = lex.get_dfa();

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
    // [20, 21, 22, 23, 30, 31, 32, 33, 34, 35, 36, 37, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67]
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

    // let lex_writer = LexWriter::new(&get_path("/resources/lex_yy.rs"), lex, );
    // lex_writer.write();
}
