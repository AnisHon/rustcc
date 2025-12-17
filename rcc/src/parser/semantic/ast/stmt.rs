use crate::parser::ast::exprs::ExprKey;
use crate::parser::semantic::ast::decl::DeclGroup;
use crate::parser::semantic::common::Ident;
use crate::parser::semantic::comp_ctx::CompCtx;
use crate::parser::semantic::sema::decl::decl_context::DeclContextRef;
use crate::types::span::{Pos, Span};
use slotmap::new_key_type;

new_key_type! {
    pub struct StmtKey;
}

#[derive(Clone, Debug)]
pub enum StmtKind {
    Expr {
        expr: Option<ExprKey>,
        semi: Pos,
    }, // exprs ;
    Decl {
        decl: DeclGroup,
    },
    Label {
        ident: Ident,
        colon: Pos,
        stmt: StmtKey,
    }, // LABEL:
    Case {
        case_span: Span,
        expr: ExprKey,
        colon: Pos,
        stmt: StmtKey,
    }, // case:
    Default {
        default: Span,
        colon: Pos,
        stmt: StmtKey,
    }, // default:
    IfElse {
        if_span: Span,
        l: Pos,
        cond: ExprKey,
        r: Pos, // if (cond) stmt else stmt
        then_stmt: StmtKey,
        else_span: Option<Span>,
        else_stmt: Option<StmtKey>,
    },
    Switch {
        switch_span: Span,
        l: Pos,
        expr: ExprKey,
        r: Pos, // switch () stmt
        body: StmtKey,
    },
    While {
        while_span: Span,
        l: Pos,
        cond: ExprKey,
        r: Pos, // while () stmt
        body: StmtKey,
    },
    DoWhile {
        do_span: Span,
        body: StmtKey,
        while_span: Span,
        l: Pos,
        cond: ExprKey,
        r: Pos, // do stmt while();
        semi: Pos,
    },
    For {
        // for ( init; cond; step ) stmt
        for_span: Span, // for
        l: Pos,         // (
        init: Option<ExprKey>,
        semi1: Pos, // init ;
        cond: Option<ExprKey>,
        semi2: Pos,            // cond ;
        step: Option<ExprKey>, // step
        r: Pos,                // )
        body: StmtKey,
    },
    Goto {
        goto_span: Span,
        ident: Ident,
        semi: Pos,
    }, // goto LABEL;
    Continue {
        continue_span: Span,
        semi: Pos,
    }, // continue;
    Break {
        break_span: Span,
        semi: Pos,
    }, // break;
    Return {
        return_span: Span,
        expr: Option<ExprKey>,
        semi: Pos,
    }, // return exprs;
    Compound {
        l: Pos,
        stmts: Vec<StmtKey>,
        r: Pos,
        context: DeclContextRef,
    }, // { ... }
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

    pub fn new_key(ctx: &mut CompCtx, kind: StmtKind, span: Span) -> StmtKey {
        ctx.insert_stmt(Self::new(kind, span))
    }
}
