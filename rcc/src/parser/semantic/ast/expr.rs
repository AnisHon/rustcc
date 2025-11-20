use crate::err::parser_error::ParserResult;
use crate::lex::types::token::Token;
use crate::lex::types::token_kind::{LiteralKind, Symbol, TokenKind};
use crate::parser::semantic::common::Ident;
use crate::parser::semantic::sema::sema_type::{Type};
use crate::types::span::{Pos, Span};
use enum_as_inner::EnumAsInner;
use std::rc::Rc;
use crate::parser::semantic::sema::expr::value_type::ValueType;

#[derive(Clone, Debug, EnumAsInner)]
pub enum ExprKind {
    DeclRef(Ident),
    Constant(LiteralKind),
    Paren { l: Pos, expr: Box<Expr>, r: Pos },
    ArraySubscript { base: Box<Expr>, l: Pos, index: Box<Expr>, r: Pos },     // a[]
    Call { base: Box<Expr>, l: Pos, params: Parameter, r: Pos },     // a()
    MemberAccess { kind: MemberAccessKind, base: Box<Expr>, op: Span, field: Symbol },       // a.b a->b
    SizeofExpr { sizeof: Span, expr: Box<Expr> },   // sizeof expr
    SizeofType { sizeof: Span, l: Pos, ty: Rc<Type>, r: Pos },   // sizeof()
    Unary { op: UnaryOp, rhs: Box<Expr> },
    Binary { lhs: Box<Expr>, op: BinOp, rhs: Box<Expr> },
    Assign { lhs: Box<Expr>, op: AssignOp, rhs: Box<Expr> },
    Cast { l: Pos, ty: Rc<Type> ,r: Pos, expr: Box<Expr> }, // (type)
    Ternary { // cond ? a : b
        cond: Box<Expr>,
        question: Pos,
        then_expr: Box<Expr>,
        colon: Pos,
        else_expr: Box<Expr>
    },
}

impl ExprKind {
    pub fn make_decl_ref(ident: Token) -> Self {
        let span = ident.span;
        let symbol = ident.kind.into_ident().unwrap();
        Self::DeclRef(Ident { symbol, span })
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
        let l = l.span.to_pos();
        let r = r.span.to_pos();
        Self::Paren{ l, expr, r }
    }

    pub fn make_index(base: Box<Expr>, l: Token, index: Box<Expr>, r: Token) -> Self {
        let l = l.span.to_pos();
        let r = r.span.to_pos();
        Self::ArraySubscript { base, l, index, r }
    }

    pub fn make_call(base: Box<Expr>, l: Token, params: Parameter, r: Token) -> Self {
        let l = l.span.to_pos();
        let r = r.span.to_pos();
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



    pub fn make_size_of_type(sizeof: Token, l: Token, ty: Rc<Type>, r: Token) -> Self {
        let sizeof = sizeof.span;
        let l = l.span.to_pos();
        let r = r.span.to_pos();
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
    
    pub fn make_cast(l: Token, ty: Rc<Type> ,r: Token, expr: Box<Expr>) -> Self {
        let l = l.span.to_pos();
        let r = r.span.to_pos();
        Self::Cast { l, ty, expr, r }
    }
    
    pub fn make_assign(lhs: Box<Expr>, op: Token, rhs: Box<Expr>) -> Self {
        let op = AssignOp::new(op);
        Self::Assign { lhs, op, rhs }
    }

    pub fn make_ternary(cond: Box<Expr>, question: Token, then_expr: Box<Expr>, colon: Token, else_expr: Box<Expr>) -> Self {
        let question = question.span.to_pos();
        let colon = colon.span.to_pos();
        Self::Ternary { cond, question, then_expr, colon, else_expr }
    }
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub ty: Rc<Type>,
    pub span: Span,
}

impl Expr {
    pub fn new(kind: ExprKind, ty: Rc<Type>, span: Span) -> Self {
        Self { kind, ty, span }
    }
    
    pub fn new_box(kind: ExprKind, ty: Rc<Type>, span: Span) -> Box<Self> {
        Box::new(Self::new(kind, ty, span))
    }

    pub fn is_lvalue(&self) -> bool {
        ValueType::value_type(self) == ValueType::LValue
    }
    
    pub fn is_int_constant(&self) -> bool {
        let ty = &self.ty;
        if ty.is_unknown() {
            todo!()
        }

        if !ty.kind.is_integer() {
            return false;
        }
        
        let constant = match &self.kind { 
            ExprKind::Constant(x) => x,
            _ => return false,
        };

        constant.is_integer() || constant.is_char()
    }

    pub fn get_int_constant(&self) -> ParserResult<u64> {
        let ty = &self.ty;
        if ty.is_unknown() {
            todo!()
        }

        if !ty.kind.is_integer() {
            // todo 类型不对
            todo!()
        }

        let constant = match &self.kind {
            ExprKind::Constant(x) => x,
            _ => {
                // 不是 constant
                todo!()
            },
        };

        let num = match constant {
            LiteralKind::Integer { value, .. } => *value,
            LiteralKind::Char { value } => {
                // char转换为num
                todo!()
            }
            _ => {
                // 不是int类型
                todo!()
            }
        };
        
        Ok(num)
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