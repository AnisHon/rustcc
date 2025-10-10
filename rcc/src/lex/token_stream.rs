use std::collections::VecDeque;
use crate::lex::types::token::Token;

pub struct TokenStream {
    token_rx: crossbeam_channel::Receiver<Token>,
    buffer: VecDeque<Token>,
}

impl TokenStream {
    
    pub fn new(token_rx: crossbeam_channel::Receiver<Token>) -> Self {
        Self {
            token_rx,
            buffer: VecDeque::new(),
        }
    }
    
    
    pub fn next(&mut self) -> Option<Token> {
        match self.buffer.pop_front() {
            None => self.token_rx.recv().map_or(None, |token| Some(token)),
            Some(x) => Some(x),
        }
    }
    
    pub fn peek(&mut self) -> Option<&Token> {
        if self.buffer.front().is_none() {
            let token = self.token_rx.recv().map_or(None, |token| Some(token))?;
            self.buffer.push_back(token);
        }
        self.buffer.front()
    }
    
    
}