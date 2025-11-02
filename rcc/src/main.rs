use rcc::compiler::c_compiler::CCompiler;

fn main() {
    let code = include_str!("../resources/example/test1.c");
    let compiler = CCompiler::new(code.to_string());
    compiler.compile();
}
