use cc::compiler::c_compiler::CCompiler;
use std::io::Cursor;

fn main() {
    const TOKEN_BOUND: usize = 4096;
    let code = include_str!("../resources/test.c");
    let compiler = CCompiler::new(Cursor::new(code), TOKEN_BOUND);
    compiler.compile();
}
