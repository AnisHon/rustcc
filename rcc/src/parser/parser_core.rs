use std::fmt::format;
use crate::err::global_err::GlobalError;
use crate::err::parser_error;
use crate::err::parser_error::{ParserError, ParserResult};
use crate::lex::token_stream::TokenStream;
use crate::lex::types::token::Token;
use crate::lex::types::token_kind::{Keyword, TokenKind};
use std::sync::mpsc;
use crate::parser::types::sema::sema_context::SemaContext;

pub struct Parser {
    pub(crate) stream: TokenStream,
    pub(crate) sema_context: SemaContext,
    pub(crate) error_tx: mpsc::Sender<GlobalError>,
}
impl Parser {
    pub fn new(stream: TokenStream, sema_context: SemaContext, error_tx: mpsc::Sender<GlobalError>) -> Parser {
        Self {
            stream,
            sema_context,
            error_tx,
        }
    }

    pub fn send_error(&mut self, err: ParserError) {
        todo!()
    }
    
    
    /// 根据条件决定是否next
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
            panic!("{error}");
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

    pub(crate) fn expect_keyword_pair(&mut self, kw1: Keyword, kw2: Keyword) -> ParserResult<Token> {
        let kind = self.stream.peek().kind;
        let expected = match kind {
            TokenKind::Keyword(k) => k == kw1 || k == kw2,
            _ => false,
        };
        if expected {
            Ok(self.stream.next())
        } else {
            let expect = format!("{}, {}", kw1.kind_str(), kw2.kind_str());
            let error_kind = parser_error::ErrorKind::Expect { expect };
            let error = self.error_here(error_kind);
            panic!("{error}");
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

    pub(crate) fn consume_keyword_pair(&mut self, kw1: Keyword, kw2: Keyword) -> Option<Token> {
        let kind = self.stream.peek().kind;
        let is_kw = match kind {
            TokenKind::Keyword(k) => k == kw1 || k == kw2,
            _ => false,
        };
        self.next_conditional(is_kw)
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

    /// (type-specifier | type-qualifier)*
    pub fn is_spec_qual(&self, token: &Token) -> bool {
        self.is_type_spec(token) || self.is_type_qual(token)
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
                    Const | Restrict | Volatile 
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
    
    pub fn is_func_spec(&self, token: &Token) -> bool {
        match token.kind {
            TokenKind::Ident(_) => self.is_type_name(token),
            TokenKind::Keyword(x) =>
                matches!(
                    x,
                    Keyword::Inline
                ),
            _ => false,
        }
    }

}