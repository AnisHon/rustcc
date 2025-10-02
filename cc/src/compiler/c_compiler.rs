use crate::lex::lex_core::{AsyncLex, Lex};
use crate::parser::parser_core::Parser;
use crate::parser::token_stream::TokenStream;
use crate::types::parser_context::ParserContext;
use std::cell::RefCell;
use std::io::Read;
use std::rc::Rc;
use std::sync::mpsc;

///
/// 编译器主流程
///
/// # Members
/// - `input`: 输入
/// - `token_bound`: token有界队列大小
///
pub struct CCompiler<R: Read> {
    input: R,
    token_bound: usize
}


impl<R: Read + Send + 'static> CCompiler<R> {
    pub fn new(input: R, token_bound: usize) -> Self {
        Self { input, token_bound }
    }


    
    /// 
    /// 编译代码，lexer --> parser --> AST
    /// 1. 前端部分lexer parser相互协作，parser提供基础的typedef定义（临时符号表），parser需要在构建CST的同时最小
    ///    限度解析作用域和typedef符号定义，lexer使用临时符号表，查询ID是否为TYPE_NAME
    /// 2. AST
    /// 
    /// 
    /// 
    pub fn compile(self) {
        
        let (token_tx, token_rx) = crossbeam_channel::bounded(self.token_bound);
        let (error_tx, error_rx) = mpsc::channel();

        let context = Rc::new(RefCell::new(ParserContext::new(error_tx.clone())));

        // lexer 是异步的
        let lex = Lex::new(self.input);
        let async_lex = AsyncLex::new(lex, token_tx, error_tx);
        async_lex.start();




        // Parser过程是同步的
        let token_stream = TokenStream::new(token_rx, Rc::clone(&context));
        let parser = Parser::new(token_stream, context);
        let ast = parser.parse().unwrap();

        for x in error_rx {
            eprintln!("{x:?}")
        }

        println!("{:?}", ast);


    }
}
