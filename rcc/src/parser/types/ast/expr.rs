use crate::lex::types::token::Token;
use crate::lex::types::token_kind::{LiteralKind, Symbol, TokenKind};
use crate::parser::types::sema::sema_expr::ValueType;
use crate::types::span::Span;

#[derive(Clone, Debug)]
pub enum ExprKind {
    DeclRef(Symbol),
    Constant(LiteralKind),
    Paren{ l: Span, expr: Box<Expr>, r: Span },
    ArraySubscript { base: Box<Expr>, l: Span, index: Box<Expr>, r: Span },     // a[]
    Call{ base: Box<Expr>, l: Span, params: Parameter, r: Span },     // a()
    MemberAccess { kind: MemberAccessKind, base: Box<Expr>, op: Span, field: Symbol },       // a.b
    SizeofExpr{ sizeof: Span, expr: Box<Expr> },   // sizeof
    SizeofType{ sizeof: Span, l: Span, ty: (), r: Span },   // sizeof
    Unary{ op: UnaryOp, rhs: Box<Expr> },
    Binary{ lhs: Box<Expr>, op: BinOp, rhs: Box<Expr> },
    Assign{ lhs: Box<Expr>, op: AssignOp, rhs: Box<Expr> },
    Cast{ l: Span, ty: () ,r: Span, expr: Box<Expr> },
    Ternary{ cond: Box<Expr>, question: Span, then_expr: Box<Expr>, colon: Span ,else_expr: Box<Expr> },
}

impl ExprKind {
    pub fn make_decl_ref(ident: Token) -> Self {
        let ident = ident.kind.into_ident().unwrap();
        Self::DeclRef(ident)
    }
    
    pub fn make_literal(token: Token) -> Self {
        let kind = match token.kind {
            TokenKind::Literal(x) => x,
            _ => unreachable!("not literal {:?}", token.kind),
        };
        Self::Constant(kind)
    }

    pub fn make_string(token: Vec<Token>) -> Self {
        let string: String = token.into_iter()
            .map(|x| x.kind.into_literal().unwrap().into_string().unwrap())
            .map(|x| x.get())
            .collect();
        let symbol = Symbol::new(&string);
        let kind = LiteralKind::String { value: symbol };
        Self::Constant(kind)
    }

    pub fn make_paren(l: Token, expr: Box<Expr>, r: Token) -> Self {
        let l = l.span;
        let r = r.span;
        Self::Paren{ l, expr, r }
    }

    pub fn make_index(base: Box<Expr>, l: Token, index: Box<Expr>, r: Token) -> Self {
        let l = l.span;
        let r = r.span;
        Self::ArraySubscript { base, l, index, r }
    }

    pub fn make_call(base: Box<Expr>, l: Token, params: Parameter, r: Token) -> Self {
        let l = l.span;
        let r = r.span;
        Self::Call { base, l, params, r }
    }

    pub fn make_dot(base: Box<Expr>, op: Token, field: Symbol) -> Self {
        let kind = match op.kind {
            TokenKind::Arrow => MemberAccessKind::Arrow,
            TokenKind::Dot => MemberAccessKind::Dot,
            _ => unreachable!("op not Arrow, Dot, {:?}", op),
        };
        let op = op.span;

        Self::MemberAccess { kind, base, op, field }
    }



    pub fn make_size_of_type(sizeof: Token, l: Token, ty: (), r: Token) -> Self {
        let sizeof = sizeof.span;
        let l = l.span;
        let r = r.span;
        Self::SizeofType { sizeof, l, ty, r }
    }

    pub fn make_size_of_expr(sizeof: Token, expr: Box<Expr>) -> Self {
        let sizeof = sizeof.span;
        Self::SizeofExpr { sizeof, expr }
    }

    pub fn make_post(lhs: Box<Expr>, op: Token) -> Self {
        let kind = match op.kind {
            TokenKind::Inc => UnaryOpKind::PostInc,
            TokenKind::Dec => UnaryOpKind::PostDec,
            _ => unreachable!("op not Inc, Dec {:?}", op),
        };
        let op = UnaryOp{kind, span: op.span};
        Self::Unary { op, rhs: lhs }
    }

    pub fn make_pre(op: Token, rhs: Box<Expr>) -> Self {
        let kind = match op.kind {
            TokenKind::Inc => UnaryOpKind::PreInc,
            TokenKind::Dec => UnaryOpKind::PreDec,
            _ => unreachable!("op not Inc, Dec {:?}", op),
        };
        let op = UnaryOp{kind, span: op.span};
        Self::Unary { op, rhs }
    }

    pub fn make_unary(op: Token, rhs: Box<Expr>) -> Self {
        let op = UnaryOp::new(op);
        Self::Unary { op, rhs }
    }

    pub fn make_binary(lhs: Box<Expr>, op: Token, rhs: Box<Expr>) -> Self {
        let op = BinOp::new(op);
        Self::Binary { lhs, op, rhs }
    }
    
    pub fn make_cast(l: Token, ty: () ,r: Token, expr: Box<Expr>) -> Self {
        let l = l.span;
        let r = r.span;
        Self::Cast { l, ty, expr, r }
    }
    
    pub fn make_assign(lhs: Box<Expr>, op: Token, rhs: Box<Expr>) -> Self {
        let op = AssignOp::new(op);
        Self::Assign { lhs, op, rhs }
    }

    pub fn make_ternary(cond: Box<Expr>, question: Token, then_expr: Box<Expr>, colon: Token, else_expr: Box<Expr>) -> Self {
        let question = question.span;
        let colon = colon.span;
        Self::Ternary { cond, question, then_expr, colon, else_expr }
    }
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

impl Expr {
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Self { kind, span }
    }
    
    pub fn new_box(kind: ExprKind, span: Span) -> Box<Self> {
        Box::new(Self::new(kind, span))
    }

    pub fn is_lvalue(&self) -> bool {
        ValueType::value_type(self) == ValueType::LValue
    }
}

#[derive(Debug, Clone)]
pub enum MemberAccessKind {
    Arrow,
    Dot
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub exprs: Vec<Box<Expr>>,
    pub commas: Vec<Span>,
}

impl Parameter {
    pub fn new() -> Self {
        Self {
            exprs: Vec::new(),
            commas: Vec::new(),
        }
    }
}

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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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