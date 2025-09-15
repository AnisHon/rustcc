use std::io::Cursor;
use cc::compiler::c_compiler::CCompiler;
use cc::types::ast::ast_nodes::*;
use cc::types::ast::decl_info::{CompleteDecl, DeclSpec, Declarator, DeclaratorChunk, ParamInfo, ParamList, TypeQual, TypeSpec};
use cc::types::ast::parser_node::ParserNode;
use cc::types::ast::struct_info::{EnumSpec, Enumerator, StructDeclarator, StructKind, StructMember, StructOrUnionSpec};
use cc::types::span::{Delim, SepList, Span};
use cc::types::token::Token;

fn main() {
    let code = include_str!("../resources/test.c");
    // let compiler = CCompiler::new(Cursor::new(code));
    // compiler.compile();
    let i = std::mem::size_of::<SepList<Enumerator>>();
    println!("{}", i);
}
