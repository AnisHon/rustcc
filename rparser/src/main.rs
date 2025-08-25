use std::fs::File;
use std::io::{BufReader, Cursor};
use rparser::file_parser::table_builder::{LRTableBuilder, TableType};
use rparser::file_parser::table_writer::TableWriter;

pub fn get_path(path: &str) -> String {
    format!("{}{}", env!("CARGO_MANIFEST_DIR"), path)
}
fn main() {

    let input = r#"%token INT

%%
expr:
    expr OP_PLUS term { $$ = $1 + $3; }
  | expr OP_MINUS term { $$ = $1 - $3; }
  | term
  |
  ;

term:
    INT
  ;
%%

"#;
    let file = File::open(get_path("/../src/clex.l")).unwrap();
    let buffer = BufReader::new(file);

    let builder = LRTableBuilder::new(TableType::LALR1, input.to_string(), buffer);
    let writer = TableWriter::new(get_path("/../src/gen/parser_yy.rs").as_str(), builder);
    writer.write();


    println!("{}", get_path("/../src/gen/parser.rs"))

}