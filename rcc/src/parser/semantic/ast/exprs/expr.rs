use crate::err::parser_error::ParserResult;
use crate::lex::types::token::Token;
use crate::lex::types::token_kind::{LiteralKind, Symbol, TokenKind};
use crate::parser::ast::exprs::{AssignOp, BinOp, UnaryOp, UnaryOpKind};
use crate::parser::ast::types::TypeKey;
use crate::parser::semantic::common::Ident;
use crate::parser::semantic::sema::expr::value_type::ValueType;
use crate::types::span::Span;
use enum_as_inner::EnumAsInner;
use slotmap::new_key_type;

new_key_type! {
    pub struct ExprKey;
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub ty: TypeKey,
    pub span: Span,
}

#[derive(Clone, Debug, EnumAsInner)]
pub enum ExprKind {
    DeclRef(Ident),
    Constant(LiteralKind),
    // Paren { l: Pos, expr: Box<Expr>, r: Pos }, no need to wrap
    ArraySubscript {
        base: Box<Expr>,
        index: Box<Expr>,
    }, // a[]
    Call {
        base: Box<Expr>,
        params: Parameter,
    }, // a()
    MemberAccess {
        kind: MemberAccessKind,
        base: Box<Expr>,
        field: Symbol,
    }, // a.b a->b
    SizeofExpr {
        expr: Box<Expr>,
    }, // sizeof exprs
    SizeofType {
        ty: TypeKey,
    }, // sizeof()
    Unary {
        op: UnaryOp,
        rhs: Box<Expr>,
    },
    Binary {
        lhs: Box<Expr>,
        op: BinOp,
        rhs: Box<Expr>,
    },
    Assign {
        lhs: Box<Expr>,
        op: AssignOp,
        rhs: Box<Expr>,
    },
    Cast {
        ty: TypeKey,
        expr: Box<Expr>,
    }, // (type)
    Ternary {
        // cond ? a : b
        cond: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
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
        let string: String = token
            .into_iter()
            .map(|x| x.kind.into_literal().unwrap().into_string().unwrap())
            .map(|x| x.get())
            .collect();
        let symbol = Symbol::new(&string);
        let kind = LiteralKind::String { value: symbol };
        Self::Constant(kind)
    }

    // pub fn make_paren(l: Token, expr: Box<Expr>, r: Token) -> Self {
    //     let l = l.span.to_pos();
    //     let r = r.span.to_pos();
    //     Self::Paren{ l, expr, r }
    // }

    pub fn make_index(base: Box<Expr>, index: Box<Expr>) -> Self {
        let l = l.span.to_pos();
        let r = r.span.to_pos();
        Self::ArraySubscript { base, index }
    }

    pub fn make_call(base: Box<Expr>, l: Token, params: Parameter, r: Token) -> Self {
        let l = l.span.to_pos();
        let r = r.span.to_pos();
        Self::Call { base, params }
    }

    pub fn make_dot(base: Box<Expr>, op: Token, field: Symbol) -> Self {
        let kind = match op.kind {
            TokenKind::Arrow => MemberAccessKind::Arrow,
            TokenKind::Dot => MemberAccessKind::Dot,
            _ => unreachable!("op not Arrow, Dot, {:?}", op),
        };
        Self::MemberAccess { kind, base, field }
    }

    pub fn make_size_of_type(sizeof: Token, l: Token, ty: TypeKey, r: Token) -> Self {
        Self::SizeofType { ty }
    }

    pub fn make_size_of_expr(sizeof: Token, expr: Box<Expr>) -> Self {
        Self::SizeofExpr { expr }
    }

    pub fn make_post(lhs: Box<Expr>, op: Token) -> Self {
        let kind = match op.kind {
            TokenKind::Inc => UnaryOpKind::PostInc,
            TokenKind::Dec => UnaryOpKind::PostDec,
            _ => unreachable!("op not Inc, Dec {:?}", op),
        };
        let op = UnaryOp {
            kind,
            span: op.span,
        };
        Self::Unary { op, rhs: lhs }
    }

    pub fn make_pre(op: Token, rhs: Box<Expr>) -> Self {
        let kind = match op.kind {
            TokenKind::Inc => UnaryOpKind::PreInc,
            TokenKind::Dec => UnaryOpKind::PreDec,
            _ => unreachable!("op not Inc, Dec {:?}", op),
        };
        let op = UnaryOp {
            kind,
            span: op.span,
        };
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

    pub fn make_cast(l: Token, ty: TypeKey, r: Token, expr: Box<Expr>) -> Self {
        Self::Cast { ty, expr }
    }

    pub fn make_assign(lhs: Box<Expr>, op: Token, rhs: Box<Expr>) -> Self {
        let op = AssignOp::new(op);
        Self::Assign { lhs, op, rhs }
    }

    pub fn make_ternary(
        cond: Box<Expr>,
        question: Token,
        then_expr: Box<Expr>,
        colon: Token,
        else_expr: Box<Expr>,
    ) -> Self {
        Self::Ternary {
            cond,
            then_expr,
            else_expr,
        }
    }
}

impl Expr {
    pub fn new(kind: ExprKind, ty: TypeKey, span: Span) -> Self {
        Self { kind, ty, span }
    }

    pub fn new_box(kind: ExprKind, ty: TypeKey, span: Span) -> Box<Self> {
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
            }
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
    Dot,
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
