use crate::lex::lex::Lex;
use crate::parser::parser::Parser;
use crate::types::token::Token;
use std::io::Read;

pub struct CCompiler<R: Read> {
    input: R
}


impl<R: Read> CCompiler<R> {
    pub fn new(input: R) -> Self {
        Self { input }
    }

    pub fn compile(self) {

        let lex = Lex::new(self.input);

        let peekable = lex.into_iter()
            .map(|x| x.unwrap())
            .filter(|x| !x.ignore())
            .chain(std::iter::once(Token::end())) // 末尾加上结束符
            .peekable();

        let parser = Parser::new(peekable);
        let cst = parser.parse().into_translation_unit().unwrap();
        println!("{:#?}", cst);
    }

}