use crate::err::global_err::GlobalError;
use crate::err::parser_error;
use crate::err::parser_error::{ParserError, ParserResult};
use crate::lex::token_stream::TokenStream;
use crate::lex::types::token::Token;
use crate::lex::types::token_kind::{Keyword, TokenKind};
use std::sync::mpsc;

pub struct Parser {
    pub(crate) stream: TokenStream,
    pub(crate) error_tx: mpsc::Sender<GlobalError>,
}
impl Parser {
    pub fn new(tokens: Vec<Token>, error_tx: mpsc::Sender<GlobalError>) -> Parser {
        Self {
            stream: TokenStream::new(tokens),
            error_tx,
        }
    }

    pub(crate) fn next_conditional(&mut self, cond: bool) -> Option<Token> {
        match cond {
            true => Some(self.stream.next()),
            false => None
        }
    }

    /// 不建议用次函数检查TokenKind下的子类型
    /// 对于Literal Ident行为未知
    pub(crate) fn check(&self, kind: TokenKind) -> bool {
        self.stream.peek().kind == kind
    }

    pub(crate) fn checks(&self, kind: &[TokenKind]) -> bool {
        let token_kind = self.stream.peek().kind;
        kind.iter().any(|kind| token_kind.eq(kind))
    }

    pub(crate) fn check_ident(&self) -> bool {
        let kind = self.stream.peek().kind;
        matches!(kind, TokenKind::Ident(_))
    }

    pub(crate) fn check_keyword(&self, keyword: Keyword) -> bool {
        let kind = self.stream.peek().kind;
        kind == TokenKind::Keyword(keyword)
    }

    /// 同上，不建议用此函数预期TokenKind下的子类型
    pub(crate) fn expects(&mut self, kinds: &[TokenKind]) -> ParserResult<Token> {
        let expected = self.checks(kinds);

        if expected {
            Ok(self.stream.next())
        } else {
            let expect: Vec<_> = kinds.iter().map(|x| x.kind_str()).collect();
            let expect = expect.join(", ");
            let found = self.stream.peek().kind.kind_str().to_owned();

            let error_kind = parser_error::ErrorKind::ExpectButFound { expect, found };
            let error = self.error_here(error_kind);
            Err(error)
        }
    }

    /// 同上，不建议用此函数预期TokenKind下的子类型
    pub(crate) fn expect(&mut self, kind: TokenKind) -> ParserResult<Token> {
        let expected = self.stream.peek().kind == kind;

        if expected {
            Ok(self.stream.next())
        } else {
            let expect = kind.kind_str().to_owned();
            let found = self.stream.peek().kind.kind_str().to_owned();

            let error_kind = parser_error::ErrorKind::ExpectButFound { expect, found };
            let error = self.error_here(error_kind);
            panic!("{error}");
            Err(error)
        }
    }

    pub(crate) fn expect_ident(&mut self) -> ParserResult<Token> {
        let expected = self.check_ident();

        if expected {
            Ok(self.stream.next())
        } else {
            let expect = "identifier".to_owned();
            let found = self.stream.peek();

            let kind = parser_error::ErrorKind::ExpectButFound { expect, found: found.kind.kind_str().to_owned() };
            let error = self.error_here(kind);
            Err(error)
        }
    }

    pub(crate) fn expect_keyword(&mut self, keyword: Keyword) -> ParserResult<Token> {
        let expected = self.check_keyword(keyword);

        if expected {
            Ok(self.stream.next())
        } else {
            let expect = keyword.kind_str().to_owned();
            let error_kind = parser_error::ErrorKind::Expect { expect };
            let error = self.error_here(error_kind);
            Err(error)
        }
    }

    /// 同上，不建议用此函数消费TokenKind下的子类型
    pub(crate) fn consumes(&mut self, kind: &[TokenKind]) -> Option<Token> {
        let is_kind = self.checks(kind);
        self.next_conditional(is_kind)
    }

    pub(crate) fn consume_pair(&mut self, kind1: TokenKind, kind2: TokenKind) -> Option<Token> {
        let kind = self.stream.peek().kind;
        let is_kind = kind == kind1 || kind == kind2;
        self.next_conditional(is_kind)
    }

    // pub(crate) fn consume_triple(&mut self, kind1: TokenKind, kind2: TokenKind, kind3: TokenKind) -> Option<Token> {
    //     let kind = self.stream.peek().kind;
    //     let is_kind = kind == kind1 || kind == kind2 || kind == kind3;
    //     self.next_conditional(is_kind)
    // }

    /// 同上，不建议用此函数消费TokenKind下的子类型
    pub(crate) fn consume(&mut self, kind: TokenKind) -> Option<Token> {
        let is_kind = self.check(kind);
        self.next_conditional(is_kind)
    }

    pub(crate) fn consume_keyword(&mut self, keyword: Keyword) -> Option<Token> {
        let is_keyword = self.check_keyword(keyword);
        self.next_conditional(is_keyword)
    }

    pub(crate) fn consume_ident(&mut self) -> Option<Token> {
        let is_ident = self.check_ident();
        self.next_conditional(is_ident)
    }

    pub(crate) fn error_here(&mut self, kind: parser_error::ErrorKind) -> ParserError {
        let span = self.stream.peek().span;
        ParserError::new(span, kind)
    }

    pub(crate) fn is_type_name(&self, token: &Token) -> bool {
        // todo
        if let TokenKind::Ident(symbol) = token.kind {
            false
        } else {
            false
        }
    }

    pub fn is_type_spec(&self, token: &Token) -> bool {
        use Keyword::*;
        match token.kind {
            TokenKind::Ident(_) => self.is_type_name(token),
            TokenKind::Keyword(x) =>
                matches!(
                    x,
                    Char | Short | Int | Long | Float | Double | Void
                    | Signed | Unsigned | Struct | Union | Enum
                ),
            _ => false,
        }
    }

    pub fn is_type_qual(&self, token: &Token) -> bool {
        use Keyword::*;
        match token.kind {
            TokenKind::Keyword(x) =>
                matches!(
                    x,
                    Const | Volatile
                ),
            _ => false,
        }
    }

    pub fn is_storage_spec(&self, token: &Token) -> bool {
        use Keyword::*;
        match token.kind {
            TokenKind::Ident(_) => self.is_type_name(token),
            TokenKind::Keyword(x) =>
                matches!(
                    x,
                    Typedef | Extern | Static | Auto | Register
                ),
            _ => false,
        }
    }
}