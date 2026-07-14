use super::sema::Sema;
use crate::err::{Diagnostic, ErrorKind};
use crate::lex::token::{Keyword, Token, TokenKind};
use crate::parser::ast::*;
use crate::source::SourceRange;

pub(crate) type PResult<T> = Result<T, Diagnostic>;

#[derive(Clone)]
pub(crate) struct DeclSpec {
    pub(crate) ty: CType,
    pub(crate) storage: StorageClass,
    pub(crate) function_specifiers: FunctionSpecifiers,
    pub(crate) alignment: Option<usize>,
}

#[derive(Clone)]
pub(crate) enum DNode {
    Name(Option<String>),
    Pointer(Box<DNode>, Qualifiers),
    Array(Box<DNode>, ArraySize),
    Function(Box<DNode>, Vec<Parameter>, bool, bool),
}

pub struct Parser {
    pub(crate) tokens: Vec<Token>,
    pub(crate) pos: usize,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) sema: Sema,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            diagnostics: vec![],
            sema: Sema::new(),
        }
    }
    pub fn parse(mut self) -> Result<TranslationUnit, Vec<Diagnostic>> {
        let mut declarations = vec![];
        while !self.at(&TokenKind::Eof) {
            match self.external_declaration() {
                Ok(mut x) => declarations.append(&mut x),
                Err(e) => {
                    self.diagnostics.push(e);
                    self.synchronize();
                }
            }
        }
        if self.diagnostics.is_empty() {
            Ok(TranslationUnit { declarations })
        } else {
            Err(self.diagnostics)
        }
    }
    pub(crate) fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }
    pub(crate) fn previous(&self) -> &Token {
        &self.tokens[self.pos - 1]
    }
    pub(crate) fn at(&self, k: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(k)
    }
    pub(crate) fn kw(&self, k: Keyword) -> bool {
        self.peek().kind == TokenKind::Keyword(k)
    }
    pub(crate) fn bump(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        if !matches!(t.kind, TokenKind::Eof) {
            self.pos += 1
        }
        t
    }
    pub(crate) fn eat(&mut self, k: &TokenKind) -> Option<Token> {
        if self.at(k) { Some(self.bump()) } else { None }
    }
    pub(crate) fn eat_kw(&mut self, k: Keyword) -> Option<Token> {
        if self.kw(k) { Some(self.bump()) } else { None }
    }
    pub(crate) fn expect(&mut self, k: &TokenKind) -> PResult<Token> {
        if self.at(k) {
            Ok(self.bump())
        } else {
            Err(self.err(format!("expected {:?}, found {:?}", k, self.peek().kind)))
        }
    }
    pub(crate) fn err(&self, msg: impl Into<String>) -> Diagnostic {
        Diagnostic::new(ErrorKind::Syntax, msg, self.peek().range)
    }
    pub(crate) fn sema_err(&self, msg: impl Into<String>, range: SourceRange) -> Diagnostic {
        Diagnostic::new(ErrorKind::Semantic, msg, range)
    }
    pub(crate) fn synchronize(&mut self) {
        while !self.at(&TokenKind::Eof) {
            if self.eat(&TokenKind::Semi).is_some() {
                break;
            }
            if self.at(&TokenKind::RBrace) {
                self.bump();
                break;
            }
            self.bump();
        }
    }
}
