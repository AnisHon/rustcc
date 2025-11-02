use crate::parser::types::ast::decl::DeclGroup;
use crate::parser::types::ast::expr::Expr;
use crate::parser::types::common::Ident;
use crate::parser::types::sema::decl::decl_context::DeclContextRef;
use crate::types::span::{Pos, Span};

#[derive(Clone, Debug)]
pub enum StmtKind {
    Expr{ expr: Option<Box<Expr>>, semi: Pos }, // expr ;
    Decl{ decl: DeclGroup },
    Label{ ident: Ident, colon: Pos, stmt: Box<Stmt> }, // LABEL: 
    Case{ case_span: Span, expr: Box<Expr>, colon: Pos, stmt: Box<Stmt> }, // case: 
    Default{ default: Span, colon: Pos, stmt: Box<Stmt> }, // default: 
    IfElse{
        if_span: Span, l: Pos, cond: Box<Expr>, r: Pos, // if (cond) stmt else stmt
        then_stmt: Box<Stmt>,
        else_span: Option<Span>,
        else_stmt: Option<Box<Stmt>>,
    },
    Switch{
        switch_span: Span, l: Pos, expr: Box<Expr>, r: Pos, // switch () stmt
        body: Box<Stmt>
    },
    While{
        while_span: Span, l: Pos, cond: Box<Expr>, r: Pos, // while () stmt
        body: Box<Stmt>
    },
    DoWhile{
        do_span: Span,
        body: Box<Stmt>,
        while_span: Span, l: Pos, cond: Box<Expr>, r: Pos, // do stmt while();
        semi: Pos
    },
    For{ // for ( init; cond; step ) stmt
        for_span: Span, // for
        l: Pos, // (
        init: Option<Box<Expr>>, semi1: Pos, // init ;
        cond: Option<Box<Expr>>, semi2: Pos, // cond ;
        step: Option<Box<Expr>>, // step
        r: Pos, // )
        body: Box<Stmt>
    },
    Goto { goto_span: Span, ident: Ident, semi: Pos }, // goto LABEL;
    Continue { continue_span: Span, semi: Pos }, // continue;
    Break { break_span: Span, semi: Pos }, // break;
    Return { return_span: Span, expr: Option<Box<Expr>>, semi: Pos }, // return expr;
    Compound { l: Pos, stmts: Vec<Box<Stmt>>, r: Pos, context: DeclContextRef }, // { ... }
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