use crate::content_manager::ContentManager;
use crate::lex::lex_core::{Lex, run_lexer};
use crate::lex::token_stream::TokenStream;
use crate::parser::ast::visitor::Visitor;
use crate::writer::ast_graph::AstGraph;
use petgraph::dot::{Config, Dot};
use std::sync::{Arc, mpsc};

///
/// 编译器主流程
///
/// # Members
/// - `input`: 输入
/// - `token_bound`: token有界队列大小
///
pub struct CCompiler {
    code: String,
}

impl CCompiler {
    pub fn new(code: String) -> Self {
        Self { code }
    }

    // ///
    // /// 编译代码，lexer --> parser --> AST
    // /// 1. 前端部分lexer parser相互协作，parser提供基础的typedef定义（临时符号表），parser需要在构建CST的同时最小
    // ///    限度解析作用域和typedef符号定义，lexer使用临时符号表，查询ID是否为TYPE_NAME
    // /// 2. AST
    // ///
    // ///
    // ///
    // pub fn compile(self) {
    //     let content_manager = Arc::new(ContentManager::new(self.code));
    //
    //     let (error_tx, error_rx) = mpsc::channel();
    //
    //     // 执行lexer
    //     let lex = Lex::new(Arc::clone(&content_manager));
    //     let tokens = run_lexer(lex, error_tx);
    //
    //
    //     let sema = Sema::new();
    //     let token_stream = TokenStream::new(tokens);
    //     let mut parser = Parser::new(token_stream, sema);
    //
    //     let mut trans_unit = parser.parse_translation_unit().unwrap();
    //     // println!("{:#?}", trans_unit);
    //
    //     let mut graph = AstGraph::new();
    //     graph.walk_translation_unit(&mut trans_unit);
    //
    //     let dot = Dot::with_config(&graph.tree, &[Config::EdgeNoLabel]);
    //     println!("{:?}", dot);
    //     // for x in error_rx {
    //     //     eprintln!("{x:?}")
    //     // }
    // }
}
