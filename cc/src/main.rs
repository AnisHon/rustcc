use cc::compiler::c_compiler::CCompiler;
use std::io::Cursor;
use cc::types::ast::struct_info::StructDeclarator;
use cc::types::ast::struct_info::{EnumSpec, Enumerator};
use cc::types::span::{Delim, SepList};

fn main() {
    const TOKEN_BOUND: usize = 4096;
    let code = include_str!("../resources/test.c");
    let compiler = CCompiler::new(Cursor::new(code), TOKEN_BOUND);
    println!("{}", size_of::<StructDeclarator>());
    // compiler.compile();
}
