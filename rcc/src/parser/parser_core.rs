use std::sync::mpsc;
use crate::err::global_err::GlobalError;
use crate::lex::token_stream::TokenStream;
use crate::lex::types::token::Token;
use crate::lex::types::token_kind::TokenKind;

pub struct Parser {
    pub(crate) stream: TokenStream,
    pub(crate) error_tx: mpsc::Sender<GlobalError>,
}
impl Parser {
    pub fn new(token_rx: crossbeam_channel::Receiver<Token>, error_tx: mpsc::Sender<GlobalError>) -> Parser {
        Self {
            stream: TokenStream::new(token_rx),
            error_tx,
        }
    }
    
    fn peek_is_ident(&mut self) -> bool {
        self.stream.peek()
            .map(|x| matches!(x.kind, TokenKind::Ident(_)))
            .unwrap_or(false)
    }
    
    
}