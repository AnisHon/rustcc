use std::iter::Peekable;
use crate::lex::lex_yy::TokenType;
use crate::types::token::Token;
use crate::parser::cst::SemanticValue;
use crate::parser::parser_yy::{exec_action, get_action, get_goto, LRAction, END_SYMBOL, EXPR_LENS, EXPR_NAMES, INIT_STATE};


pub struct Parser<I, ValueType = SemanticValue>
where
    I: Iterator<Item =Token>,
{
    state_stack: Vec<usize>,         // 状态栈
    value_stack: Vec<ValueType>, // 语义栈
    iter: Peekable<I>
}

impl<I> Parser<I>
where
    I: Iterator<Item =Token>,
{
    pub fn new(iter: I) -> Self {
        let iter = iter.peekable();
        Self {
            state_stack: vec![INIT_STATE],  // 必须压入初始状态
            value_stack: vec![SemanticValue::default()], // 保持栈平衡，可去
            iter
        }
    }

    pub fn parse(mut self) -> SemanticValue {
        while let Some(token) = self.iter.peek() {
            let state = *self.state_stack.last().unwrap();
            let token_id = token.typ as usize;
            let action = get_action(state, token_id);

            match action {
                LRAction::Reduce(expr) => self.reduce(expr),
                LRAction::Accept(expr) => {self.reduce(expr);break;},
                LRAction::Shift(state) => self.shift(state),
                LRAction::Error => self.error(),
            }
        }

        self.value_stack.pop().unwrap()
    }

    fn reduce(&mut self, expr: usize) {
        let expr_len = EXPR_LENS[expr];
        let pop_idx = self.state_stack.len() - expr_len;

        self.state_stack.drain(pop_idx..); // 推出 状态栈
        let state = *self.state_stack.last().unwrap(); // 当前状态
        self.state_stack.push(get_goto(state, expr).unwrap()); // 压入状态栈

        let values: Vec<_> = self.value_stack.drain(pop_idx..).collect(); // 推出 语义栈
        let value = exec_action(expr, values); // 执行语义
        self.value_stack.push(value); // 压入语义栈
    }

    fn shift(&mut self, state: usize) {
        let token = self.iter.next().unwrap();
        let value = SemanticValue::Token(token);
        self.state_stack.push(state);
        self.value_stack.push(value);
    }

    fn error(&mut self) { // 错误恢复没做
        println!("{:?}", self.iter.peek());
        panic!("error");

    }

}




#[test]
fn test() {
    
    // int main();
    let mut x: Vec<_> = [TokenType::KeywordInt, TokenType::Id, TokenType::Lparen, TokenType::Rparen, TokenType::Semicolon].into_iter().map(|x| x as usize).collect();
    x.push(END_SYMBOL);
    println!("{:?}", x);


}

