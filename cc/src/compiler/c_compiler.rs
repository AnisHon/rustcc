use std::cell::RefCell;
use crate::lex::lex_core::Lex;
use crate::parser::parser_core::Parser;
use crate::types::token::Token;
use std::io::Read;
use std::rc::Rc;
use crate::types::symbol_table::SymbolTable;

pub struct CCompiler<R: Read> {
    input: R
}


impl<R: Read> CCompiler<R> {
    pub fn new(input: R) -> Self {
        Self { input }
    }


    
    /// 
    /// 编译代码，lexer --> parser --> CST --> AST 
    /// 1. 前端部分lexer parser相互协作，parser提供基础的typedef定义（临时符号表），parser需要在构建CST的同时最小
    ///    限度解析作用域和typedef符号定义，lexer使用临时符号表，查询ID是否为TYPE_NAME
    /// 2. AST
    /// 
    /// 
    /// 
    pub fn compile(self) {
        
        let symbol_table: Rc<RefCell<SymbolTable<()>>> = Rc::new(RefCell::new(SymbolTable::new())); // 临时符号表，仅用于查询符号

        let lex = Lex::new(self.input, Rc::clone(&symbol_table));
        let iter = lex.into_iter()
            .chain(std::iter::once(Token::end_token())); // 末尾加上结束符

        let parser = Parser::new(iter, symbol_table);
        let cst = parser.parse().unwrap();
        
        println!("{:?}", cst);




    }
}
