use std::fs::File;
use std::io::{BufReader};
use rparser::file_parser::config_reader::token_from_lexer;
use rparser::file_parser::table_builder::{LRTableBuilder, TableType};
use rparser::file_parser::table_writer::TableWriter;

pub fn get_path(path: &str) -> String {
    format!("{}{}", env!("CARGO_MANIFEST_DIR"), path)
}
fn main() {
    
    let buffer = BufReader::new(File::open(get_path("/../src/clex.l")).unwrap());
    let vec = token_from_lexer(buffer);

    let input = include_str!("../../src/parser.y");
    

    let builder = LRTableBuilder::new(TableType::LALR1, input.to_string(), vec);
    let writer = TableWriter::new(get_path("/../src/gen/parser_yy.rs").as_str(), builder);
    writer.write();


    println!("{}", get_path("/../src/gen/parser.rs"))

}