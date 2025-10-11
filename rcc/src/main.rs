use rcc::compiler::c_compiler::CCompiler;

fn main() {
    let code = include_str!("../resources/test.c");
    let compiler = CCompiler::new(code.to_string());
    compiler.compile();
}
