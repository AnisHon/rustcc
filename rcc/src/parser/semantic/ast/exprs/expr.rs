use crate::err::parser_error::{ParserError, ParserResult};
use crate::lex::types::token::Token;
use crate::lex::types::token_kind::{LiteralKind, Symbol, TokenKind};
use crate::parser::ast::exprs::{AssignOp, BinOp, UnaryOp, UnaryOpKind};
use crate::parser::ast::{ExprKey, TypeKey};
use crate::parser::semantic::common::Ident;
use crate::parser::semantic::sema::expr::value_type::ValueType;
use crate::types::span::Span;
use enum_as_inner::EnumAsInner;
use num_bigint::BigInt;

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub ty: TypeKey,
    pub span: Span,
    pub value: Option<BigInt>,
}

#[derive(Clone, Debug, EnumAsInner)]
pub enum ExprKind {
    DeclRef(Ident),
    Literal(LiteralKind), // 字符串
    // Paren { l: Pos, expr: ExprKey, r: Pos }, no need to wrap
    ArraySubscript {
        base: ExprKey,
        index: ExprKey,
    }, // a[]
    Call {
        base: ExprKey,
        params: Parameter,
    }, // a()
    MemberAccess {
        kind: MemberAccessKind,
        base: ExprKey,
        field: Symbol,
    }, // a.b a->b
    SizeofExpr {
        expr: ExprKey,
    }, // sizeof exprs
    SizeofType {
        ty: TypeKey,
    }, // sizeof()
    Unary {
        op: UnaryOp,
        rhs: ExprKey,
    },
    Binary {
        lhs: ExprKey,
        op: BinOp,
        rhs: ExprKey,
    },
    Assign {
        lhs: ExprKey,
        op: AssignOp,
        rhs: ExprKey,
    },
    Cast {
        ty: TypeKey,
        expr: ExprKey,
    }, // (type)
    Ternary {
        // cond ? a : b
        cond: ExprKey,
        then_expr: ExprKey,
        else_expr: ExprKey,
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
        Self::Literal(kind)
    }

    pub fn make_string(token: Vec<Token>) -> Self {
        let string: String = token
            .into_iter()
            .map(|x| x.kind.into_literal().unwrap().into_string().unwrap())
            .map(|x| x.get())
            .collect();
        let symbol = Symbol::new(&string);
        let kind = LiteralKind::String { value: symbol };
        Self::Literal(kind)
    }

    // pub fn make_paren(l: Token, expr: ExprKey, r: Token) -> Self {
    //     let l = l.span.to_pos();
    //     let r = r.span.to_pos();
    //     Self::Paren{ l, expr, r }
    // }

    pub fn make_index(base: ExprKey, index: ExprKey) -> Self {
        Self::ArraySubscript { base, index }
    }

    pub fn make_call(base: ExprKey, l: Token, params: Parameter, r: Token) -> Self {
        let l = l.span.to_pos();
        let r = r.span.to_pos();
        Self::Call { base, params }
    }

    pub fn make_dot(base: ExprKey, op: Token, field: Symbol) -> Self {
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

    pub fn make_size_of_expr(sizeof: Token, expr: ExprKey) -> Self {
        Self::SizeofExpr { expr }
    }

    pub fn make_post(lhs: ExprKey, op: Token) -> Self {
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

    pub fn make_pre(op: Token, rhs: ExprKey) -> Self {
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

    pub fn make_unary(op: Token, rhs: ExprKey) -> Self {
        let op = UnaryOp::new(op);
        Self::Unary { op, rhs }
    }

    pub fn make_binary(lhs: ExprKey, op: Token, rhs: ExprKey) -> Self {
        let op = BinOp::new(op);
        Self::Binary { lhs, op, rhs }
    }

    pub fn make_cast(l: Token, ty: TypeKey, r: Token, expr: ExprKey) -> Self {
        Self::Cast { ty, expr }
    }

    pub fn make_assign(lhs: ExprKey, op: Token, rhs: ExprKey) -> Self {
        let op = AssignOp::new(op);
        Self::Assign { lhs, op, rhs }
    }

    pub fn make_ternary(
        cond: ExprKey,
        question: Token,
        then_expr: ExprKey,
        colon: Token,
        else_expr: ExprKey,
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
        Self {
            kind,
            ty,
            span,
            value: None,
        }
    }

    pub fn is_lvalue(&self) -> bool {
        ValueType::value_type(self) == ValueType::LValue
    }

    pub fn should_int_constant(&self) -> ParserResult<BigInt> {
        // 不是 constant 出错
        let num = self
            .value
            .clone()
            .ok_or(ParserError::not_int_constant(self.span))?;

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
    pub exprs: Vec<ExprKey>,
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
