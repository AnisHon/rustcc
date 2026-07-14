use crate::ap::ap_float::APFloat;
use crate::ap::ap_int::APInt;
use crate::parser::ast::types::QualType;
use crate::types::lex::token::Token;
use crate::types::lex::token_kind::{LiteralKind, Symbol, TokenKind};
use crate::types::parser::ast::exprs::{AssignOp, BinOp, UnaryOp, UnaryOpKind};
use crate::types::parser::ast::{ExprKey, TypeKey};
use crate::types::parser::common::Ident;
use crate::types::span::Span;
use enum_as_inner::EnumAsInner;

pub enum Constant {
    Integer { value: APInt },
    Float { value: APFloat },
    String { value: Vec<u8> }, // 0 结尾 u8 数组，长度一定 >= 1
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub ty: TypeKey,
    pub span: Span,
}

impl Expr {
    pub fn new(kind: ExprKind, ty: TypeKey, span: Span) -> Self {
        Self { kind, ty, span }
    }
}
#[derive(Clone, Debug, EnumAsInner)]
pub enum ExprKind {
    DeclRef(Ident),
    Literal(LiteralKind), // 字符串
    // Paren { l: Pos, expr: ExprKey, r: Pos }, no need to wrap
    ArraySubscript(ArraySubscriptExpr), // a[]
    Call(CallExpr),                     // a()
    MemberAccess(MemberAccessExpr),     // a.b a->b
    Sizeof(SizeofExpr),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Assign(AssignExpr),
    Cast(CastExpr),
    Ternary(TernaryExpr),
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

    pub fn make_index(base: ExprKey, index: ExprKey) -> Self {
        Self::ArraySubscript(ArraySubscriptExpr { base, index })
    }

    pub fn make_call(base: ExprKey, params: ParamsExpr) -> Self {
        Self::Call(CallExpr { base, params })
    }

    pub fn make_dot(base: ExprKey, op: Token, field: Symbol) -> Self {
        let kind = match op.kind {
            TokenKind::Arrow => MemberAccessKind::Arrow,
            TokenKind::Dot => MemberAccessKind::Dot,
            _ => unreachable!("op not Arrow, Dot, {:?}", op),
        };
        Self::MemberAccess(MemberAccessExpr { kind, base, field })
    }

    pub fn make_size_of_type(ty: QualType) -> Self {
        Self::Sizeof(SizeofExpr::OfType(ty))
    }

    pub fn make_size_of_expr(expr: ExprKey) -> Self {
        Self::Sizeof(SizeofExpr::OfExpr(expr))
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
        Self::Unary(UnaryExpr { op, rhs: lhs })
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
        Self::Unary(UnaryExpr { op, rhs })
    }

    pub fn make_unary(op: Token, rhs: ExprKey) -> Self {
        let op = UnaryOp::new(op);
        Self::Unary(UnaryExpr { op, rhs })
    }

    pub fn make_binary(lhs: ExprKey, op: Token, rhs: ExprKey) -> Self {
        let op = BinOp::new(op);
        Self::Binary(BinaryExpr { lhs, op, rhs })
    }

    pub fn make_cast(ty: QualType, expr: ExprKey) -> Self {
        Self::Cast(CastExpr { ty, expr })
    }

    pub fn make_assign(lhs: ExprKey, op: Token, rhs: ExprKey) -> Self {
        let op = AssignOp::new(op);
        Self::Assign(AssignExpr { lhs, op, rhs })
    }

    pub fn make_ternary(cond: ExprKey, then_expr: ExprKey, else_expr: ExprKey) -> Self {
        Self::Ternary(TernaryExpr {
            cond,
            then_expr,
            else_expr,
        })
    }
}

/// 数组访问表达式
#[derive(Clone, Debug)]
pub struct ArraySubscriptExpr {
    pub base: ExprKey,
    pub index: ExprKey,
}

/// 函数调用表达式
#[derive(Clone, Debug)]
pub struct CallExpr {
    pub base: ExprKey,
    pub params: ParamsExpr,
}

/// 成员访问表达式
#[derive(Clone, Debug)]
pub struct MemberAccessExpr {
    pub kind: MemberAccessKind,
    pub base: ExprKey,
    pub field: Symbol,
}

#[derive(Debug, Clone)]
pub enum MemberAccessKind {
    Arrow,
    Dot,
}
/// sizeof 表达式 包括 sizeof type 和 sizeof expression
#[derive(Clone, Debug)]
pub enum SizeofExpr {
    OfExpr(ExprKey),
    OfType(QualType),
}

/// 一元运算表达式
#[derive(Clone, Debug)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub rhs: ExprKey,
}

/// 二元运算表达式
#[derive(Clone, Debug)]
pub struct BinaryExpr {
    pub lhs: ExprKey,
    pub op: BinOp,
    pub rhs: ExprKey,
}

/// 赋值表达式
#[derive(Clone, Debug)]
pub struct AssignExpr {
    pub lhs: ExprKey,
    pub op: AssignOp,
    pub rhs: ExprKey,
}

/// 类型转换表达式
#[derive(Clone, Debug)]
pub struct CastExpr {
    pub ty: QualType,
    pub expr: ExprKey,
}

/// 三元运算表达式
#[derive(Clone, Debug)]
pub struct TernaryExpr {
    pub cond: ExprKey,
    pub then_expr: ExprKey,
    pub else_expr: ExprKey,
}

/// 表达式调用参数
#[derive(Debug, Clone, Default)]
pub struct ParamsExpr {
    pub exprs: Vec<ExprKey>,
}
