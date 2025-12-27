use crate::lex::types::token::Token;
use crate::lex::types::token_kind::TokenKind;
use crate::types::span::Span;

#[derive(Debug, Clone)]
pub enum AssignOpKind {
    Assign,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    ShlEq,
    ShrEq,
    AmpEq,
    CaretEq,
    PipeEq,
}

#[derive(Debug, Clone)]
pub struct AssignOp {
    pub kind: AssignOpKind,
    pub span: Span,
}

impl AssignOp {
    pub fn new(token: Token) -> Self {
        use AssignOpKind::*;
        let kind = match token.kind {
            TokenKind::Assign => Assign,
            TokenKind::PlusEq => PlusEq,
            TokenKind::MinusEq => MinusEq,
            TokenKind::StarEq => StarEq,
            TokenKind::SlashEq => SlashEq,
            TokenKind::PercentEq => PercentEq,
            TokenKind::ShlEq => ShlEq,
            TokenKind::ShrEq => ShrEq,
            TokenKind::AmpEq => AmpEq,
            TokenKind::PipeEq => PipeEq,
            TokenKind::CaretEq => CaretEq,
            _ => unreachable!("not assign operator {:?}", token.kind),
        };
        let span = token.span;
        Self { kind, span }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BinOpKind {
    Plus, Minus, Mul, Div, Mod,
    BitAnd, BitOr, BitXor,
    Xor, Shl, Shr,
    Lt, Gt, Eq, Ne, Le, Ge,
    And, Or, Comma,
}

#[derive(Debug, Clone)]
pub struct BinOp {
    pub kind: BinOpKind,
    pub span: Span,
}

impl BinOp {
    pub fn new(token: Token) -> Self {
        use BinOpKind::*;
        let kind = match token.kind {
            TokenKind::Plus => Plus,
            TokenKind::Minus => Minus,
            TokenKind::Star => Mul,
            TokenKind::Slash => Div,
            TokenKind::Percent => Mod,
            TokenKind::Amp => BitAnd,
            TokenKind::Pipe => BitOr,
            TokenKind::Caret => Xor,
            TokenKind::Shl => Shl,
            TokenKind::Shr => Shr,
            TokenKind::Lt => Lt,
            TokenKind::Gt => Gt,
            TokenKind::Eq => Eq,
            TokenKind::Ne => Ne,
            TokenKind::Le => Le,
            TokenKind::Ge => Ge,
            TokenKind::And => And,
            TokenKind::Or => Or,
            TokenKind::Comma => Comma,
            _ => unreachable!("not binary operator {:?}", token.kind),
        };
        let span = token.span;
        Self { kind, span }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOpKind {
    AddrOf,
    Deref,
    Plus,
    Minus,
    Not,
    BitNot,
    PostInc,
    PostDec,
    PreInc,
    PreDec,
}

#[derive(Debug, Clone)]
pub struct UnaryOp {
    pub kind: UnaryOpKind,
    pub span: Span,
}

impl UnaryOp {
    pub fn new(token: Token) -> Self {
        use UnaryOpKind::*;
        let kind = match token.kind {
            TokenKind::Amp => AddrOf,
            TokenKind::Star => Deref,
            TokenKind::Plus => Plus,
            TokenKind::Minus => Minus,
            TokenKind::Bang => Not,
            TokenKind::Tilde => BitNot,
            _ => unreachable!("not unary operator {:?}", token.kind),
        };
        let span = token.span;
        Self { kind, span }
    }
}