use std::cell::RefCell;
use std::rc::Rc;
use crate::types::lex::token::Token;
use crate::types::lex::token_kind::TokenKind;
use crate::types::symbol_table::SymbolTable;

///
/// Token流对象，可以实现对Token的预处理，做为parser与lexer之间桥梁
/// todo 待实现
#[allow(dead_code)]
pub struct TokenStream {
    rx: crossbeam_channel::Receiver<Token>,
    symbol_table: Rc<RefCell<SymbolTable<()>>>
}

#[allow(dead_code)]
impl TokenStream  {
    pub fn new(rx: crossbeam_channel::Receiver<Token>, symbol_table: Rc<RefCell<SymbolTable<()>>>) -> Self {
        Self {
            rx,
            symbol_table
        }
    }

    fn next_token(&mut self) -> Option<Token> {
        // 当channel失效结束
        let mut token = self.rx.recv().ok()?;

        match token.kind {
            TokenKind::ID => {
                let content = token.value.as_string().unwrap();
                // 被声明为typename
                if self.symbol_table.borrow().lookup(content).is_some() {
                    token.kind = TokenKind::TypeName;
                };
            }
            _ => {}
        }

        Some(token)
    }

}

impl Iterator for TokenStream {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

