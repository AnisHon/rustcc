use crate::lex::types::token::Token;
use crate::types::span::Span;

///
/// TokenStream
/// 
/// # Members
/// - `pos`: 当前指针位置，可以保证永远不越界
/// - `tokens`: lexer输出的token数组
/// - `last_pos`: 上次计算span的位置
/// - `mark_pos`: 保存回溯位置的栈
/// 
pub struct TokenStream {
    pos: usize,
    tokens: Vec<Token>,
    last_pos: usize,
    mark_pos: Vec<usize>,
}

impl TokenStream {
    
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            pos: 0,
            last_pos: 0,
            tokens,
            mark_pos: Vec::new(),
        }
    }

    pub fn next(&mut self) -> Token {
        let token = self.tokens[self.pos];
        if self.pos < self.tokens.len() - 1 { // 非Eof移动 
            self.pos += 1;
        }
        token
    }
    
    pub fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }
    
    /// peek 下一个token
    pub fn peek_next(&self) -> &Token {
        self.tokens.get(self.pos + 1).unwrap_or(self.tokens.last().unwrap())
    }

    // /// 保存当前位置，用于回溯
    // pub fn mark(&mut self) {
    //     self.mark_pos.push(self.pos);
    // }
    
    // pub fn pop_mark(&mut self) {
    //     self.mark_pos.pop();
    // }
    
    // /// 回溯位置，如果没有回溯点会panic
    // pub fn rewind(&mut self) {
    //     self.pos = self.mark_pos.pop().expect("Pos stack is empty");
    // }
    
    // /// 最后一个token
    // pub fn last(&self) -> &Token {
    //     self.tokens.last().unwrap() // 一定存在最后的token（EOF）
    // }

    /// 当前token的span
    pub fn span(&mut self) -> Span {
        self.peek().span
    }

    /// 上一个token的span，如果当前位置是0会出错
    pub fn prev_span(&mut self) -> Span {
        assert!(self.pos > 0);
        self.tokens[self.pos - 1].span
        
    }
}