use cc::compiler::c_compiler::CCompiler;
use cc::types::ast::ast_nodes::*;
use std::io::Cursor;

fn main() {
    let code = include_str!("../resources/test.c");
    let compiler = CCompiler::new(Cursor::new(code));
    compiler.compile();
    let sz = std::mem::size_of::<TranslationUnit>();
    println!("{}", sz);
}
