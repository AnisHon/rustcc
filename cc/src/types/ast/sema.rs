//!
//! Sema是semantic的缩写，这是生成语义节点的地方，这里的生成的节点严格对应AST树
//! Sema函数会做语义检查，类型检查，错误处理 和 错误恢复
//!

use crate::types::ast::ast_nodes::*;
use crate::types::ast::decl_info::{DeclSpec, Declarator, DeclChunk, TypeQual};
use crate::types::ast::parser_node::ParserNode;
use crate::types::lex::token::Token;
use crate::types::lex::token_kind::TokenKind;
use crate::types::span::UnwrapSpan;
use std::mem;
use crate::types::ast::initializer::InitDeclarator;

impl TranslationUnit {
    pub fn make_translation_unit(ext_decl: ExternalDeclaration) -> ParserNode {
        TranslationUnit { ext_decls: vec![ext_decl], }.into()
    }

    pub fn insert_ext_decl(mut translation_unit: TranslationUnit, ext_decl: ExternalDeclaration) {
        translation_unit.unwrap_span().merge_self(&ext_decl.unwrap_span());
        translation_unit.ext_decls.push(ext_decl);
    }
}

impl FunctionDefinition {
    pub fn make(
        decl_spec: Option<DeclSpec>,
        declarator: Declarator,
        decl_list: Option<DeclList>,  // 老式类型声明
        stmt: CompoundStatement
    ) -> ParserNode {
        todo!() // 函数定义
    }
}

impl BlockItem {
    
    pub fn make_decl(decl: DeclStmt) -> ParserNode {
        Self::Declaration(decl).into()
    }
    
    pub fn make_stmt(stmt: Statement) -> ParserNode {
        Self::Statement(stmt).into()
    }
    
    pub fn make_list(block_item: BlockItem) -> ParserNode {
        BlockItemList::from([block_item]).into()
    }
    
    pub fn push(mut list: BlockItemList, block_item: BlockItem) -> ParserNode {
        list.push(block_item);
        list.into()
    }
}


impl Type {

    pub fn make_type(_chunk: Vec<DeclChunk>) -> Self {
        todo!() // todo 未实现
    }
}


impl Qualifiers {

    pub fn new(is_const: bool, is_volatile: bool) -> Self {
        Self {
            is_const,
            is_volatile
        }
    }

    pub fn set(&mut self, type_qual: TypeQual) {
        match type_qual {
            TypeQual::Const(_x) => {
                if self.is_const {
                    panic!("duplicate 'const'");
                } else {
                    self.is_const = true;
                }
            }
            TypeQual::Volatile(_x) => {
                if self.is_volatile {
                    panic!("duplicate 'volatile'");
                } else {
                    self.is_volatile = true;
                }
            }
        }
    }
}

impl Default for Qualifiers {
    fn default() -> Self {
        Self::new(false, false)
    }
}

impl CompoundStatement {
    pub fn make(lbrace: Token, list: Option<BlockItemList>, rbrace: Token) -> CompoundStatement {
        Self {
            lbrace: lbrace.span,
            list,
            rbrace: rbrace.span
        }
    }
}

impl Statement {
    /// constexpr应该会被归并成为一个常量表达式，最终被计算
    pub fn make_case(constexpr: Expression, stmt: Statement) -> ParserNode {
        let (constant, span) = match constexpr.kind {
            ExpressionKind::Literal(constant) => {
                let span = constant.span;
                (constant, span)
            },
            _ => unreachable!()
        };

        let value = match constant.kind {
            ConstantKind::Int(x) => x,
            ConstantKind::Char(x) => x as i64,
            _ => panic!("Clangd: Integer constant expression must have integer type, not '{:?}'", constexpr.ty)
        };

        let kind = StatementKind::Case { value, stmt: Box::new(stmt) };
        Statement::new(kind, span).into()
    }

    pub fn make_expression(expr: Option<Box<Expression>>, semi: Token) -> ParserNode {
        let span = semi.span;
        let span = expr.as_ref().map(|x| span.merge(&x.span)).unwrap_or(span);

        let kind = StatementKind::Expression(expr);
        Statement::new(kind, span).into()
    }

    pub fn make_if(if_token: Token, cond: Expression, then_stmt: Statement, else_stmt: Option<Statement>) -> ParserNode {
        let span = if_token.span;
        let span = match &else_stmt {
            None => span,
            Some(x) => span.merge(&x.span)
        };

        let kind = StatementKind::If { cond: Box::new(cond), then_stmt: Box::new(then_stmt), else_stmt: else_stmt.map(Box::new) };
        Statement::new(kind, span).into()
    }

    pub fn make_switch(switch_token: Token, cond: Expression, body: Statement) -> ParserNode {
        let span = switch_token.span.merge(&body.span);
        let kind = StatementKind::Switch { cond: Box::new(cond), body: Box::new(body) };
        Statement::new(kind, span).into()
    }
    pub fn make_while(while_token: Token, cond: Expression, body: Statement, rparen: Option<Token>) -> ParserNode {
        let span = while_token.span.merge(&body.span);
        let span = match rparen {
            None => span,
            Some(x) => x.span.merge(&span)
        };
        let kind = match while_token.kind {
            TokenKind::KeywordWhile => StatementKind::While { cond: Box::new(cond), body: Box::new(body) },
            TokenKind::KeywordDo => StatementKind::DoWhile { cond: Box::new(cond), body: Box::new(body) },
            _ => unreachable!()
        };
        Statement::new(kind, span).into()
    }

    pub fn make_for(for_token: Token, init_opt: Option<Expression>, cond_opt: Option<Expression>, step_opt: Option<Expression>, body: Statement) -> ParserNode {
        let span = for_token.span.merge(&body.span);
        let kind = StatementKind::For { init: init_opt.map(Box::new), cond: cond_opt.map(Box::new), step: step_opt.map(Box::new), body: Box::new(body) };
        Statement::new(kind, span).into()
    }

    /// 第一个token是goto
    pub fn make_goto(goto: Token, label: Token) -> ParserNode {
        let goto_span = goto.span;
        let label_span = label.span;

        let span = goto_span.merge(&label_span);
        let label = label.value.into_string().unwrap();
        let kind = StatementKind::Goto { label };
        Statement::new(kind, span).into()
    }


    pub fn make_continue_break(token: Token) -> ParserNode {
        let span = token.span;
        let kind = match token.kind {
            TokenKind::KeywordContinue => StatementKind::Continue,
            TokenKind::KeywordBreak => StatementKind::Break,
            _ => unreachable!()
        };

        Statement::new(kind, span).into()
    }

    /// 第一个token是return
    pub fn make_return(ret: Token, expr: Option<Expression>) -> ParserNode {
        let ret_span = ret.span;
        let span = match &expr {
            None => ret_span,
            Some(expr) => ret_span.merge(&expr.span)
        };

        let kind = StatementKind::Return(expr.map(Box::new));
        Statement::new(kind, span).into()
    }

    pub fn make_compound(stmt: CompoundStatement) -> ParserNode {
        let span = stmt.unwrap_span();
        let kind = StatementKind::Compound(stmt);
        Statement::new(kind, span).into()
    }
}

impl Expression {

    pub fn make_literal(constant: Constant) -> ParserNode {
        let span = constant.span;
        let kind = ExpressionKind::Literal(constant);
        Box::new(Expression::new(kind, None, span)).into()
    }

    pub fn make_id(token: Token) -> ParserNode {
        let span = token.span;
        let name = token.value.into_string().unwrap();
        let kind = ExpressionKind::Id { name, decl_ref: None };
        Box::new(Expression::new(kind, None, span)).into()
    }

    /// 最后的token是 arr[...] <-这个字符，用来精确确定位置
    pub fn make_array_access(base: Expression, index: Expression, token: Token) -> ParserNode {
        let span = base.span.merge(&token.span);
        let kind = ExpressionKind::ArrayAccess { base: Box::new(base), index: Box::new(index) };
        Box::new(Expression::new(kind, None, span)).into()
    }

    /// 最后的token是 foo(...) <-这个字符，用来精确确定位置
    pub fn make_call(func: Expression, args: Vec<Expression>, token: Token) -> ParserNode {
        let span = func.span.merge(&token.span);
        let kind = ExpressionKind::Call {func: Box::new(func), args};

        Box::new(Expression::new(kind, None, span)).into()
    }

    pub fn make_field_access(base: Expression, field: Token) -> ParserNode {
        let span = base.span.merge(&field.span);
        let field = field.value.into_string().unwrap();
        let kind = ExpressionKind::FieldAccess { base: Box::new(base), field };

        Box::new(Expression::new(kind, None, span)).into()
    }

    pub fn make_arrow(base: Expression, field: Token) -> ParserNode {
        let span = base.span.merge(&field.span);
        let field = field.value.into_string().unwrap();
        let kind = ExpressionKind::Arrow { base: Box::new(base), field };

        Box::new(Expression::new(kind, None, span)).into()

    }

    ///
    /// 构建 前后置 ++ --
    /// # Arguments
    /// expr:
    /// token:
    /// post: 是否是后置
    ///
    pub fn make_update(expr: Expression, token: Token, post: bool) -> ParserNode {
        let span = token.span.merge(&expr.span);

        let kind = match (token.kind, post) {
            (TokenKind::OpDec, true) => ExpressionKind::PostDec,
            (TokenKind::OpDec, false) => ExpressionKind::PreDec,
            (TokenKind::OpInc, true) => ExpressionKind::PostInc,
            (TokenKind::OpInc, false) => ExpressionKind::PreInc,
            _ => unreachable!()
        };

        let kind = kind(Box::new(expr));
        Box::new(Expression { kind, ty: None, span }).into()
    }

    pub fn make_unary(token: Token, expr: Expression) -> ParserNode {
        let token_span = token.span;
        let span = token_span.merge(&expr.span);

        let kind = match token.kind {
            TokenKind::OpBitand => UnaryOpKind::AddressOf,
            TokenKind::OpTimes => UnaryOpKind::Deref,
            TokenKind::OpPlus => UnaryOpKind::Plus,
            TokenKind::OpMinus => UnaryOpKind::Minus,
            TokenKind::OpBitNot => UnaryOpKind::BitNot,
            TokenKind::OpNot => UnaryOpKind::LogicalNot,
            _ => unreachable!()
        };

        let op = UnaryOp::new(kind, token_span);


        let expr_kind = ExpressionKind::Unary {op, expr: Box::new(expr)};

        Box::new(Expression::new(expr_kind, None, span)).into()
    }

    /// 第一个token是sizeof的值 -> sizeof expr
    pub fn make_sizeof_expr(_sizeof: Token, expr: Expression) -> ParserNode {
        let span = expr.span;
        let kind = ExpressionKind::SizeofExpr(Box::new(expr));

        Box::new(Expression::new(kind, None, span)).into()
    }

    /// 第一个token是sizeof的值 -> sizeof(type) <- 第二个是第二个括号
    pub fn make_sizeof_type(typ: Type, rparen: Token) -> ParserNode {
        todo!()
    }


    /// 第一个token 是类型转换的第一个括号-> (X)X
    pub fn make_cast(token: Token, typ: Type, expr: Expression) -> ParserNode {
        todo!();
       
    }

    pub fn make_binary(lhs: Expression, token: Token, rhs: Expression) -> ParserNode {
        let span = lhs.span.merge(&rhs.span);
        let span_token = token.span;


        let op_kind: BinaryOpKind = token.try_into().unwrap();
        let op = BinaryOp::new(op_kind, span_token);

        let kind = ExpressionKind::Binary { lhs: Box::new(lhs), op, rhs: Box::new(rhs) };

        Box::new(Expression::new(kind, None, span)).into()
    }


    pub fn make_conditional(cond: Box<Expression>, then_expr: Box<Expression>, else_expr: Box<Expression>) -> ParserNode {
        let span = cond.span.merge(&else_expr.span);
        let kind = ExpressionKind::Conditional {
            cond,
            then_expr,
            else_expr
        };

        Box::new(Expression::new(kind, None, span)).into()
    }

    pub fn make_assign(lhs: Box<Expression>, token: Token, rhs: Box<Expression>) -> ParserNode {
        if lhs.is_rvalue() {
            panic!("Cannot assign to rvalue");
        }
        let span_token = token.span;

        let op_kind: AssignOpKind = token.try_into().unwrap();
        let span = lhs.span.merge(&rhs.span);
        let op = AssignOp::new(op_kind, span_token);

        let kind = ExpressionKind::Assign { lhs, op, rhs };
        Box::new(Expression { kind, ty: None, span }).into()
    }

    pub fn make_comma(mut exprs: Vec<Expression>, expr: Expression) -> ParserNode {
        exprs.push(expr);
        exprs.into()
    }

}

impl Constant {
    pub fn make(token: Token) -> ParserNode {
        Constant::try_from(token).unwrap().into()
    }

    pub fn insert_str(mut constant: Constant, token: Token) -> ParserNode {
        let token_str = token.value.as_string().unwrap();
        let token_span = token.span;
        let string = match &mut constant.kind {
            ConstantKind::String(str) => str,
            _ => unreachable!(),
        };

        constant.span.merge_self(&token_span);
        string.push_str(token_str);
        constant.into()
    }

}



impl Decl {
    pub fn make(decl_spec: DeclSpec, init_decl: InitDeclarator, semi: Token) -> ParserNode {
        todo!()
    }

    pub fn make_list(decl: Box<Decl>) -> ParserNode {
        DeclList::from([decl]).into()
    }

    pub fn push(mut decl_list: DeclList, decl: Box<Decl>) -> ParserNode {
        decl_list.push(decl);
        decl_list.into()
    }

}