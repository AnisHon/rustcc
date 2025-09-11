//!
//! Sema是semantic的缩写，这是生成语义节点的地方，这里的生成的节点严格对应AST树
//! Sema函数会做语义检查，类型检查，错误处理 和 错误恢复
//!

use crate::lex::lex_yy::TokenType;
use crate::types::ast::ast_nodes::*;
use crate::types::ast::parser_node::ParserNode;
use crate::parser::span::Span;
use crate::types::token::{Token, TokenValue};

impl TranslationUnit {
    pub fn make_translation_unit(ext_decl: ExternalDeclaration) -> ParserNode {
        let span = ext_decl.unwrap_span();
        TranslationUnit {
            ext_decls: vec![ext_decl],
            span
        }.into()
    }

    pub fn insert_ext_decl(mut translation_unit: TranslationUnit, ext_decl: ExternalDeclaration) {
        translation_unit.span.merge_self(&ext_decl.unwrap_span());
        translation_unit.ext_decls.push(ext_decl);
    }
}


impl ExternalDeclaration {

    pub fn make_func(func_def: FunctionDefinition) -> ParserNode {
        let span = func_def.span;
        Self::Function(func_def, span).into()
    }

    pub fn make_variable(decl: Declaration) -> ParserNode {
        let span = decl.span;
        Self::Variable(decl, span).into()
    }

    pub fn unwrap_span(&self) -> Span {
        match self {
            ExternalDeclaration::Function(_, x) => *x,
            ExternalDeclaration::Variable(_, x) => *x
        }
    }
}

impl Type {
    pub fn unwarp_span(&self) -> Span {
        match self {
            Type::Void(x) => *x,
            Type::Integer { span, .. } => *span,
            Type::Floating { span, .. } => *span,
            Type::Pointer(_, x) => *x,
            Type::Array { span, .. } => *span,
            Type::Function { span, .. } => *span,
            Type::Struct { span, .. } => *span,
            Type::Union { span, .. } => *span,
            Type::Enum { span, .. } => *span,
            Type::NamedType { span, .. } => *span,
        }
    }

    pub fn set_span(&mut self, set: Span) {
        match self {
            Type::Void(span)
            | Type::Integer { span, .. }
            | Type::Floating { span, .. }
            | Type::Pointer(_, span)
            | Type::Array { span, .. }
            | Type::Function { span, .. }
            | Type::Struct { span, .. }
            | Type::Union { span, .. }
            | Type::Enum { span, .. }
            | Type::NamedType { span, .. } => {
                *span = set;
            }
        }
    }
}

impl Qualifiers {

    pub fn new(is_const: bool, is_volatile: bool) -> Self {
        Self {
            is_const,
            is_volatile
        }
    }
    pub fn make(decl: Option<Qualifiers>, qual: Token) -> ParserNode {
        let result = match (decl, qual.as_type().unwrap()) {
            (None, TokenType::KeywordConst) => Qualifiers::new(true, false),
            (Some(mut decl), TokenType::KeywordConst) => {decl.set_const(); decl},
            (None, TokenType::KeywordVolatile) => Qualifiers::new(false, true),
            (Some(mut decl), TokenType::KeywordVolatile) => {decl.set_volatile(); decl},
            _ => unreachable!(),
        };

        result.into()
    }

    pub fn set_const(&mut self) {
        // todo 检查
        self.is_const = true;
    }

    pub fn set_volatile(&mut self) {
        // todo 检查
        self.is_volatile = true;
    }
}


impl Statement {
    /// constexpr应该会被归并成为一个常量表达式，最终被计算
    pub fn make_case(constexpr: Expression, stmt: Statement) -> ParserNode {
        let (constant, span) = match constexpr.kind {
            ExpressionKind::Literal(constant, span) => (constant, span),
            _ => unreachable!()
        };

        let value = match constant {
            Constant::Int(x, _) => x,
            Constant::Char(x, _) => x as i64,
            _ => panic!("Clangd: Integer constant expression must have integer type, not '{:?}'", constexpr.ty)
        };

        Statement::Case {value, stmt: Box::new(stmt), span}.into()
    }

    pub fn make_expression(expr: Option<Expression>, semi: Token) -> ParserNode {
        let span = Span::from_token(&semi);
        let span = expr.as_ref().map(|x| span.merge(&x.span)).unwrap_or(span);

        Statement::Expression(expr, span).into()
    }

    pub fn make_if(if_token: Token, cond: Expression, then_stmt: Statement, else_stmt: Option<Statement>) -> ParserNode {
        let span = Span::from_token(&if_token);
        let span = match &else_stmt {
            None => span,
            Some(x) => span.merge(&x.unwrap_span())
        };

        Statement::If { cond, then_stmt: Box::new(then_stmt), else_stmt: else_stmt.map(Box::new), span }.into()
    }

    pub fn make_switch(switch_token: Token, cond: Expression, body: Statement) -> ParserNode {
        let span = Span::from_token(&switch_token).merge(&body.unwrap_span());
        Statement::Switch { cond, body: Box::new(body), span }.into()
    }
    pub fn make_while(while_token: Token, cond: Expression, body: Statement, rparen: Option<Token>) -> ParserNode {
        let span = Span::from_token(&while_token).merge(&body.unwrap_span());
        let span = match rparen {
            None => span,
            Some(x) => Span::from_token(&x).merge(&span)
        };
        let result = match while_token.as_type().unwrap() {
            TokenType::KeywordWhile => Statement::While { cond, body: Box::new(body), span },
            TokenType::KeywordDo => Statement::DoWhile { cond, body: Box::new(body), span },
            _ => unreachable!()
        };
        result.into()
    }

    pub fn make_for(for_token: Token, init_opt: Option<Expression>, cond_opt: Option<Expression>, step_opt: Option<Expression>, body: Statement) -> ParserNode {
        let span = Span::from_token(&for_token).merge(&body.unwrap_span());
        Statement::For { init: init_opt, cond: cond_opt, step: step_opt, body: Box::new(body), span }.into()
    }

    /// 第一个token是goto
    pub fn make_goto(goto: Token, label: Token) -> ParserNode {
        let goto_span = Span::from_token(&goto);
        let label_span = Span::from_token(&label);

        let span = goto_span.merge(&label_span);
        let label = label.value.into_string().unwrap();
        Statement::Goto { label, span }.into()
    }


    pub fn make_continue_break(token: Token) -> ParserNode {
        let span = Span::from_token(&token);
        let result = match token.as_type().unwrap() {
            TokenType::KeywordContinue => Statement::Continue(span),
            TokenType::KeywordBreak => Statement::Break(span),
            _ => unreachable!()
        };

        result.into()
    }

    /// 第一个token是return
    pub fn make_return(ret: Token, expr: Option<Expression>) -> ParserNode {
        let ret_span = Span::from_token(&ret);
        let span = match &expr {
            None => ret_span,
            Some(expr) => ret_span.merge(&expr.span)
        };

        Statement::Return(expr, span).into()
    }
}

impl Expression {

    pub fn make_literal(constant: Constant) -> ParserNode {
        let span = constant.unwrap_span();
        let kind = ExpressionKind::Literal(constant, span.clone());
        Expression { kind, ty: None, span }.into()
    }

    pub fn make_id(token: Token) -> ParserNode {
        let name = token.value.into_string().unwrap();
        ExpressionKind::Id {name, decl_ref: None}.into()
    }

    /// 最后的token是 arr[...] <-这个字符，用来精确确定位置
    pub fn make_array_access(base: Expression, index: Expression, token: Token) -> ParserNode {
        let span = base.span.merge(&Span::from_token(&token));
        let kind = ExpressionKind::ArrayAccess { base: Box::new(base), index: Box::new(index) };
        Expression { kind, ty: None, span }.into()
    }

    /// 最后的token是 foo(...) <-这个字符，用来精确确定位置
    pub fn make_call(func: Expression, args: Vec<Expression>, token: Token) -> ParserNode {
        let span = func.span.merge(&Span::from_token(&token));
        let kind = ExpressionKind::Call {func: Box::new(func), args};

        Expression { kind, ty: None, span }.into()
    }

    pub fn make_field_access(base: Expression, field: Token) -> ParserNode {
        let span = base.span.merge(&Span::from_token(&field));
        let field = field.value.into_string().unwrap();
        let kind = ExpressionKind::FieldAccess { base: Box::new(base), field };

        Expression { kind, ty: None, span }.into()
    }

    pub fn make_arrow(base: Expression, field: Token) -> ParserNode {
        let span = base.span.merge(&Span::from_token(&field));
        let field = field.value.into_string().unwrap();
        let kind = ExpressionKind::Arrow { base: Box::new(base), field };

        Expression { kind, ty: None, span }.into()

    }

    ///
    /// 构建 前后置 ++ --
    /// # Arguments
    /// expr:
    /// token:
    /// post: 是否是后置
    ///
    pub fn make_update(expr: Expression, token: Token, post: bool) -> ParserNode {
        let span = Span::from_token(&token).merge(&expr.span);

        let kind = match (token.as_type().unwrap(), post) {
            (TokenType::OpDec, true) => ExpressionKind::PostDec,
            (TokenType::OpDec, false) => ExpressionKind::PreDec,
            (TokenType::OpInc, true) => ExpressionKind::PostInc,
            (TokenType::OpInc, false) => ExpressionKind::PreInc,
            _ => unreachable!()
        };

        let kind = kind(Box::new(expr));
        Expression { kind, ty: None, span }.into()
    }

    pub fn make_unary(token: Token, expr: Expression) -> ParserNode {
        let token_span = Span::from_token(&token);
        let span = token_span.merge(&expr.span);
        let op = match token.as_type().unwrap() {
            TokenType::OpBitand => UnaryOp::AddressOf(token_span),
            TokenType::OpTimes => UnaryOp::Deref(token_span),
            TokenType::OpPlus => UnaryOp::Plus(token_span),
            TokenType::OpMinus => UnaryOp::Minus(token_span),
            TokenType::OpBitNot => UnaryOp::BitNot(token_span),
            TokenType::OpNot => UnaryOp::LogicalNot(token_span),
            _ => unreachable!()
        };
        let kind = ExpressionKind::Unary {op, expr: Box::new(expr)};

        Expression { kind , ty: None, span }.into()
    }

    /// 第一个token是sizeof的值 -> sizeof expr
    pub fn make_sizeof_expr(sizeof: Token, expr: Expression) -> ParserNode {
        let span = expr.span.clone();
        let kind = ExpressionKind::SizeofExpr(Box::new(expr));

        Expression { kind, ty: None, span }.into()
    }

    /// 第一个token是sizeof的值 -> sizeof(type) <- 第二个是第二个括号
    pub fn make_sizeof_type(sizeof: Token, typ: Type, rparen: Token) -> ParserNode {
        let span = Span::from_token(&sizeof).merge(&Span::from_token(&rparen));
        let kind = ExpressionKind::SizeofType(typ);

        Expression { kind, ty: None, span }.into()
    }


    /// 第一个token 是类型转换的第一个括号-> (X)X
    pub fn make_cast(token: Token, typ: Type, expr: Expression) -> ParserNode {
        let span = Span::from_token(&token).merge(&expr.span);
        let kind = ExpressionKind::Cast { ty: typ, expr: Box::new(expr) };

        Expression { kind, ty: None, span }.into()
    }

    pub fn make_binary(lhs: Expression, token: Token, rhs: Expression) -> ParserNode {
        let span = lhs.span.merge(&rhs.span);
        let span_token = Span::from_token(&token);


        let op = match token.as_type().unwrap() {
            TokenType::OpPlus => BinaryOp::Add(span_token),
            TokenType::OpMinus => BinaryOp::Sub(span_token),
            TokenType::OpTimes => BinaryOp::Mul(span_token),
            TokenType::OpDivide => BinaryOp::Div(span_token),
            TokenType::OpMod => BinaryOp::Mod(span_token),
            TokenType::OpLShift => BinaryOp::Shl(span_token),
            TokenType::OpRShift => BinaryOp::Shr(span_token),
            TokenType::OpLt => BinaryOp::Lt(span_token),
            TokenType::OpGt => BinaryOp::Gt(span_token),
            TokenType::OpLe => BinaryOp::Le(span_token),
            TokenType::OpGe => BinaryOp::Ge(span_token),
            TokenType::OpEq => BinaryOp::Eq(span_token),
            TokenType::OpNe => BinaryOp::Ne(span_token),
            TokenType::OpBitand => BinaryOp::BitAnd(span_token),
            TokenType::OpXor => BinaryOp::BitXor(span_token),
            TokenType::OpBitor => BinaryOp::BitOr(span_token),
            TokenType::OpAnd => BinaryOp::LogicalAnd(span_token),
            TokenType::OpOr => BinaryOp::LogicalOr(span_token),
            _ => unreachable!()
        };
        let kind = ExpressionKind::Binary { lhs: Box::new(lhs), op, rhs: Box::new(rhs) };

        Expression { kind, ty: None, span }.into()
    }


    pub fn make_conditional(cond: Expression, then_expr: Expression, else_expr: Expression) -> ParserNode {
        let span = cond.span.merge(&else_expr.span);
        let kind = ExpressionKind::Conditional {
            cond: Box::new(cond),
            then_expr: Box::new(then_expr),
            else_expr: Box::new(else_expr)
        };

        Expression { kind, ty: None, span }.into()
    }

    pub fn make_assign(lhs: Expression, token: Token, rhs: Expression) -> ParserNode {
        if lhs.is_rvalue() {
            panic!("Cannot assign to rvalue");
        }
        let span_token = Span::from_token(&token);

        let op = match token.as_type().unwrap() {
            TokenType::OpAssign => AssignOp::Assign(span_token),
            TokenType::OpMulAssign => AssignOp::MulAssign(span_token),
            TokenType::OpDivAssign => AssignOp::DivAssign(span_token),
            TokenType::OpModAssign => AssignOp::ModAssign(span_token),
            TokenType::OpAddAssign => AssignOp::AddAssign(span_token),
            TokenType::OpSubAssign => AssignOp::SubAssign(span_token),
            TokenType::OpLShiftAssign => AssignOp::ShlAssign(span_token),
            TokenType::OpRShiftAssign => AssignOp::ShrAssign(span_token),
            TokenType::OpAndAssign => AssignOp::AndAssign(span_token),
            TokenType::OpOrAssign => AssignOp::OrAssign(span_token),
            _ => unreachable!(),
        };
        let span = lhs.span.merge(&rhs.span);
        let kind = ExpressionKind::Assign { lhs: Box::new(lhs), op, rhs: Box::new(rhs) };
        Expression { kind, ty: None, span }.into()
    }

    pub fn make_comma(mut exprs: Vec<Expression>, expr: Expression) -> ParserNode {
        exprs.push(expr);
        exprs.into()
    }


}

impl Constant {
    pub fn make(token: Token) -> ParserNode {
        let span = Span::from_token(&token);

        let constant = match token.value {
            TokenValue::Number { value, .. } => Self::Int(value as i64, span),
            TokenValue::Float(value) => Self::Float(value, span),
            TokenValue::String(value) => Self::String(value, span),
            TokenValue::Char(value) => Self::Char(value, span),
            TokenValue::Other => unreachable!(),
        };

        constant.into()
    }

    pub fn insert_str(mut constant: Constant, token: Token) -> ParserNode {
        let token_str = token.value.as_string().unwrap();
        let token_span = Span::from_token(&token);
        let (str, span) = match &mut constant {
            Constant::String(str, span) => (str, span),
            _ => unreachable!(),
        };

        span.merge_self(&token_span);
        str.push_str(token_str);
        constant.into()
    }

    pub fn unwrap_span(&self) -> Span {
        match self {
            Constant::Int(_, x) => *x,
            Constant::Float(_, x) => *x,
            Constant::Char(_, x) => *x,
            Constant::String(_, x) => *x,
        }
    }

    pub fn get_type(&self) -> Type {
        let span = self.unwrap_span();
        match self {
            Constant::Int(_, _) => Type::Integer {signed: true, size: IntegerSize::Int, span},
            Constant::Float(_, _) => Type::Floating {size: FloatSize::Float, span},
            Constant::Char(_, _) => Type::Integer {signed: true, size: IntegerSize::Char, span},
            Constant::String(x, _) => Type::string_type(x.len() as u64, span),
        }
    }
}
