use std::io::Cursor;
use cc::compiler::c_compiler::CCompiler;



fn main() {
    let code = include_str!("../resources/test.c");
    let compiler = CCompiler::new(Cursor::new(code));
    compiler.compile();

}
