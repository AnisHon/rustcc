use crate::types::lex::token::Token;
use crate::types::lex::token_kind::TokenKind;
use crate::types::parser_context::ParserContext;
use std::cell::RefCell;
use std::rc::Rc;

///
/// Token流对象，可以实现对Token的预处理，做为parser与lexer之间桥梁
/// todo 待实现
#[allow(dead_code)]
pub struct TokenStream {
    rx: crossbeam_channel::Receiver<Token>,
    peeked: Option<Token>, // 缓存一个
    context: Rc<RefCell<ParserContext>>,
}

#[allow(dead_code)]
impl TokenStream {
    pub fn new(
        rx: crossbeam_channel::Receiver<Token>,
        context: Rc<RefCell<ParserContext>>,
    ) -> Self {
        Self {
            rx,
            peeked: None,
            context,
        }
    }

    fn try_typename(&self, token: &mut Token) {
        if matches!(token.kind, TokenKind::ID) {
            let content = token.value.as_string().unwrap();
            // 被声明为typename
            // todo typename检查
            // if self.symbol_table.borrow().lookup(content).is_some() {
            //     token.kind = TokenKind::TypeName;
            // };
        }
    }

    fn next_token(&mut self) -> Option<Token> {
        // 当channel失效结束
        let mut token = self.rx.recv().ok()?;
        self.try_typename(&mut token);
        Some(token)
    }

    pub fn next(&mut self) -> Option<Token> {
        if let Some(token) = self.peeked.take() {
            return Some(token);
        }
        self.next_token()
    }

    pub fn peek(&mut self) -> Option<&Token> {
        if self.peeked.is_none() {
            self.peeked = Some(self.next_token()?);
        }
        self.peeked.as_ref()
    }

    /// peek缓存后可能导致不一致问题，该函数强制使其同步
    pub fn sync(&mut self) {
        let mut token = match self.peeked.take() {
            None => return, // 没有缓存无需同步
            Some(x) => x,
        };

        // 尝试同步
        self.try_typename(&mut token);
        // 同步后写回
        self.peeked = Some(token);
    }
}
