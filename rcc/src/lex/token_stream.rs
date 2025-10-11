use crate::lex::types::token::Token;

pub struct TokenStream {
    pos: usize,
    tokens: Vec<Token>,
    mark_pos: Vec<usize>,
}

impl TokenStream {
    
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            pos: 0,
            tokens,
            mark_pos: Vec::new(),
        }
    }

    pub fn next(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let token = self.tokens[self.pos];
            self.pos += 1;
            Some(token)
        } else {
            None
        }
    }
    
    pub fn peek(&mut self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    /// 保存当前位置，用于回溯
    fn mark(&mut self) {
        self.mark_pos.push(self.pos);
    }
    
    /// 回溯位置，如果没有回溯点会panic
    fn rewind(&mut self) {
        self.pos = self.mark_pos.pop().expect("Pos stack is empty");
    }

    /// 最后一个token
    pub fn last(&mut self) -> &Token {
        self.tokens.last().unwrap() // 一定存在最后的token（EOF）
    }
}