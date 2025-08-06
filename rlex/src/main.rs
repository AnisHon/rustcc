use rlex::rlexer::lex_reader::LexReader;
use rlex::rlexer::lex_writer::LexWriter;
use rlex::rlexer::lexer::Lexer;

fn main() {
    let lex = LexReader::new(r"E:\Project02\rust\rustcc\rlex\resources\clex.l")
        .read_from_file()
        .unwrap();
    // lex.iter().for_each(|x| println!("{:?}", x));

    println!("{}", lex.len());

    let lex = Lexer::new(lex);

    // let char_class_set = lex.get_char_class_set();
    let dfa = lex.get_dfa();

    // let mut state = dfa.get_init_state();
    // for chr in "\"asdfasdf\\*\"".chars() {
    //     let c = char_class_set.find_char(chr);
    //     println!("{:?}", dfa.get_meta(state));
    //     state = match dfa.find_next(state, c) {
    //         None => panic!("Wrong"),
    //         Some(x) => x,
    //     };
    // }
    //
    // println!("{:?}", dfa.get_meta(state));
    // println!("size: {} x {}", dfa.size(), dfa.get_stride());

    let mut sum = 0;
    for x in 0..dfa.size() {
        sum += dfa.get_symbols(x).len()
    }
    println!("edges: {}", sum);

    let lex_writer = LexWriter::new(
        r"E:\Project02\rust\rustcc\rlex\resources\lex_yy.rs".to_string(),
        lex,
    );
    lex_writer.write();
}
