use rparser::file_parser::table_builder::{LRTableBuilder, TableType};
use rparser::file_parser::table_writer::TableWriter;

pub fn get_path(path: &str) -> String {
    format!("{}{}", env!("CARGO_MANIFEST_DIR"), path)
}
fn main() {

    let input = include_str!("../../src/parser_test.y");
    

    let builder = LRTableBuilder::new(TableType::LALR1, input.to_string());
    let writer = TableWriter::new(
        get_path("/../src/gen/parser_yy.rs").as_str(),
        get_path("/../src/gen/decl_yy.rs").as_str(),
        builder
    );
    writer.write();


    // println!("{}", get_path("/../src/gen/parser.rs"))

}