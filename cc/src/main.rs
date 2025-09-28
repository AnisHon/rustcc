use cc::compiler::c_compiler::CCompiler;
use std::io::Cursor;
use cc::types::ast::ast_nodes::{Decl, Type};
use cc::types::ast::func_info::{ParamDecl, ParamList};
use cc::types::ast::initializer::{InitDeclList, InitDeclarator, InitInfo};
use cc::types::ast::parser_node::ParserNode;
use cc::types::ast::type_info::{CompleteDecl, StructDecl, StructDeclarator};
use cc::types::ast::type_info::{EnumSpec, Enumerator};
use cc::types::span::{Delim, SepList};

fn main() {
    const TOKEN_BOUND: usize = 4096;
    let code = include_str!("../resources/test.c");
    let compiler = CCompiler::new(Cursor::new(code), TOKEN_BOUND);
    println!("{}", size_of::<CompleteDecl>());
    // compiler.compile();
}
