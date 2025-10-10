use crate::lex::lex_core::{run_async, Lex};
use crate::types::parser_context::ParserContext;
use std::cell::RefCell;
use std::fs;
use std::io::Read;
use std::rc::Rc;
use std::sync::{mpsc, Arc};
use crate::content_manager::ContentManager;

///
/// 编译器主流程
///
/// # Members
/// - `input`: 输入
/// - `token_bound`: token有界队列大小
///
pub struct CCompiler<> {
    token_bound: usize,
    file_path: String,
}


impl CCompiler<> {
    pub fn new(file_path: String, token_bound: usize) -> Self {
        Self { file_path, token_bound }
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
        let content = fs::read_to_string(&self.file_path).unwrap();
        let content_manager = Arc::new(ContentManager::new(content));
        
        let (token_tx, token_rx) = crossbeam_channel::bounded(self.token_bound);
        let (error_tx, error_rx) = mpsc::channel();
        
        // lexer 是异步的
        let lex = Lex::new(Arc::clone(&content_manager));
        run_async(lex, token_tx, error_tx);
        
        
        
        for x in error_rx {
            eprintln!("{x:?}")
        }


    }
}
