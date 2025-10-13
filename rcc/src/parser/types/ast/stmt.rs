use crate::lex::types::token_kind::Symbol;
use crate::parser::types::ast::expr::Expr;
use crate::parser::types::common::Ident;
use crate::types::span::Span;

#[derive(Clone, Debug)]
pub enum StmtKind {
    Expr{ expr: Option<Box<Expr>>, semi: Span },
    Decl{ },
    Label{ ident: Ident, colon: Span, stmt: Box<Stmt> },
    Case{ case_span: Span, expr: Box<Expr>, colon: Span, stmt: Box<Stmt> },
    Default{ default: Span, colon: Span, stmt: Box<Stmt> },
    IfElse{
        if_span: Span, l: Span, cond: Box<Expr>, r: Span,
        then_stmt: Box<Stmt>,
        else_span: Option<Span>,
        else_stmt: Option<Box<Stmt>>,
    },
    Switch{
        switch_span: Span, l: Span, cond: Box<Expr>, r: Span,
        body: Box<Stmt>
    },
    While{
        while_span: Span, l: Span, cond: Box<Expr>, r: Span,
        body: Box<Stmt>
    },
    DoWhile{
        do_span: Span,
        body: Box<Stmt>,
        while_span: Span, l: Span, cond: Box<Expr>, r: Span,
        semi: Span
    },
    For{ // for ( init; cond; step ) stmt
        for_span: Span, // for
        l: Span, // (
        init: Option<Box<Expr>>, semi1: Span, // init ;
        cond: Option<Box<Expr>>, semi2: Span, // cond ;
        step: Option<Box<Expr>>, // step
        r: Span, // )
        body: Box<Stmt>
    },
    Goto{ goto_span: Span, ident: Ident, semi: Span },
    Continue{ continue_span: Span, semi: Span },
    Break{ break_span: Span, semi: Span },
    Return{ return_span: Span, expr: Option<Box<Expr>>, semi: Span },
    Compound{ l: Span, stmts: Vec<Box<Stmt>>, r: Span },
}

#[derive(Clone, Debug)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

impl Stmt {
    pub fn new(kind: StmtKind, span: Span) -> Self {
        Stmt { kind, span }
    }

    pub fn new_box(kind: StmtKind, span: Span) -> Box<Self> {
        Box::new(Self::new(kind, span))
    }
}