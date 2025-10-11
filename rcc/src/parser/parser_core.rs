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
    pub(crate) fn check(&mut self, kind: TokenKind) -> bool {
        self.stream.peek().kind == kind
    }

    pub(crate) fn checks(&mut self, kind: &[TokenKind]) -> bool {
        let token_kind = self.stream.peek().kind;
        kind.iter().all(|kind| token_kind.eq(kind))
    }

    pub(crate) fn check_ident(&mut self) -> bool {
        let kind = self.stream.peek().kind;
        matches!(kind, TokenKind::Ident(_))
    }

    pub(crate) fn check_keyword(&mut self, keyword: Keyword) -> bool {
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
            let found = self.stream.peek();

            let error_kind = parser_error::ErrorKind::ExpectButFound { expect, found: found.kind.kind_str().to_owned() };
            Err(ParserError::new(found.span, error_kind))
        }
    }

    /// 同上，不建议用此函数预期TokenKind下的子类型
    pub(crate) fn expect(&mut self, kind: TokenKind) -> ParserResult<Token> {
        let expected = self.stream.peek().kind == kind;

        if expected {
            Ok(self.stream.next())
        } else {
            let expect = kind.kind_str().to_owned();
            let found = self.stream.peek();

            let error_kind = parser_error::ErrorKind::ExpectButFound { expect, found: found.kind.kind_str().to_owned() };
            Err(ParserError::new(found.span, error_kind))
        }
    }

    pub(crate) fn expect_ident(&mut self) -> ParserResult<Token> {
        let expected = self.check_ident();

        if expected {
            Ok(self.stream.next())
        } else {
            let expect = "identifier".to_owned();
            let found = self.stream.peek();

            let error_kind = parser_error::ErrorKind::ExpectButFound { expect, found: found.kind.kind_str().to_owned() };
            Err(ParserError::new(found.span, error_kind))
        }
    }

    // pub(crate) fn expect_keyword(&mut self, keyword: Keyword) -> ParserResult<Token> {
    //     let expected = self.stream.peek().kind == TokenKind::Keyword(keyword);
    //
    //     if expected {
    //         Ok(self.stream.next())
    //     } else {
    //         let expect = kind.kind_str();
    //         let found = self.stream.peek();
    //
    //         let error_kind = parser_error::ErrorKind::Expect { expect, found: found.kind.kind_str() };
    //         Err(ParserError::new(found.span, error_kind))
    //     }
    // }

    /// 同上，不建议用此函数消费TokenKind下的子类型
    pub(crate) fn consumes(&mut self, kind: &[TokenKind]) -> Option<Token> {
        let is_kind = self.checks(kind);
        self.next_conditional(is_kind)
    }

    /// 同上，不建议用此函数消费TokenKind下的子类型
    pub(crate) fn consume(&mut self, kind: TokenKind) -> Option<Token> {
        let is_kind = self.check(kind);
        self.next_conditional(is_kind)
    }

    pub(crate) fn consume_keyword(&mut self, keyword: Keyword) -> Option<Token> {
        let is_keyword = self.check_keyword(keyword);
        self.next_conditional(is_keyword)
    }

    pub(crate) fn error_here(&mut self, kind: parser_error::ErrorKind) -> ParserError {
        let span = self.stream.peek().span;
        ParserError::new(span, kind)
    }

    /// 匹配带间隔符的列表，不负责消费最后的结束符，不允许存在尾随分隔符
    ///
    /// # Arguments
    /// - `sep`: 分割字符
    /// - `end`: 结束字符
    /// - `parse_elem`: 解析函数
    ///
    pub(crate) fn parse_sep_list<T>(
        &mut self,
        sep: TokenKind,
        end: TokenKind,
        parse_elem: impl Fn(&mut Self) -> ParserResult<T>,
    ) -> ParserResult<Vec<T>> {
        let mut elems = Vec::new();
        if self.check(end) {
            return Ok(elems);
        }

        loop {
            elems.push(parse_elem(self)?);
            if let Some(_tok) = self.consume(sep) {
                todo!()
            } else if self.check(end) {
                break
            } else {
                let kind = parser_error::ErrorKind::Expect {
                    expect: end.kind_str().to_owned()
                };
                return Err(self.error_here(kind));
            }
        }
        Ok(elems)
    }
}