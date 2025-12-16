//!
//! Sema是semantic的缩写，这是生成语义节点的地方，这里的生成的节点严格对应AST树
//! Sema函数会做语义检查，类型检查，错误处理 和 错误恢复
//!

use crate::types::ast::nodes::*;
use crate::types::ast::decl_info::{DeclSpec, Declarator, DeclChunk, TypeQual, DeclChunkList, DeclChunkKind, PointerChunk};
use crate::types::ast::sematic_value::SemanticValue;
use crate::types::lex::token::Token;
use crate::types::lex::token_kind::TokenKind;
use crate::types::span::UnwrapSpan;
use std::mem;
use crate::types::ast::nodes;
use crate::types::ast::initializer::{InitDeclList, InitDeclarator};
use crate::types::ast::seme::flat_chunks;

impl TranslationUnit {
    pub fn make_translation_unit(ext_decl: ExternalDeclaration) -> SemanticValue {
        TranslationUnit { ext_decls: vec![ext_decl], }.into()
    }

    pub fn insert_ext_decl(mut translation_unit: TranslationUnit, ext_decl: ExternalDeclaration) -> SemanticValue {
        translation_unit.unwrap_span().merge_self(&ext_decl.unwrap_span());
        translation_unit.ext_decls.push(ext_decl);
        translation_unit.into()
    }
}

impl ExternalDeclaration {
    pub fn make_func(def: Box<FunctionDefinition>) -> SemanticValue {
        Self::Function(def).into()
    }

    pub fn make_decl(decl: DeclStmt) -> SemanticValue {
        Self::Declaration(decl).into()
    }
}

impl FunctionDefinition {
    pub fn make(
        decl_spec: Option<DeclSpec>,
        declarator: Declarator,
        decl_list: Option<DeclList>,  // 老式类型声明
        stmt: Statement
    ) -> SemanticValue {
        todo!() // 函数定义
    }
}

impl BlockItem {
    
    pub fn make_decl(decl: DeclStmt) -> SemanticValue {
        Self::Declaration(decl).into()
    }
    
    pub fn make_stmt(stmt: Statement) -> SemanticValue {
        Self::Statement(stmt).into()
    }
    
    pub fn make_list(block_item: BlockItem) -> SemanticValue {
        BlockItemList::from([block_item]).into()
    }
    
    pub fn push(mut list: BlockItemList, block_item: BlockItem) -> SemanticValue {
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

impl Statement {

    pub fn make_label(ident: Token, stmt: Statement) -> SemanticValue {
        let span = ident.span.merge(&stmt.span);
        let label = ident.value.into_string().unwrap();
        let kind = StatementKind::Labeled { label, stmt: Box::new(stmt) };
        Self::new(kind, span).into()
    }


    /// constexpr应该会被归并成为一个常量表达式，最终被计算
    pub fn make_case(kw_case: Token, constexpr: Box<Expression>, stmt: Statement) -> SemanticValue {
        let span = kw_case.span.merge(&stmt.span);
        let constant = match constexpr.kind {
            ExpressionKind::Literal(constant) => constant,
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

    pub fn make_default(kw_default: Token, stmt: Statement) -> SemanticValue {
        let span = kw_default.span.merge(&stmt.span);
        let kind = StatementKind::Default {stmt: Box::new(stmt) };
        Statement::new(kind, span).into()
    }

    pub fn make_expression(expr: Option<Box<Expression>>, semi: Token) -> SemanticValue {
        let span = semi.span;
        let span = expr.as_ref().map(|x| span.merge(&x.span)).unwrap_or(span);

        let kind = StatementKind::Expression(expr);
        Statement::new(kind, span).into()
    }

    pub fn make_if(if_token: Token, cond: Box<Expression>, then_stmt: Statement, else_stmt: Option<Statement>) -> SemanticValue {
        let span = if_token.span;
        let span = match &else_stmt {
            None => span,
            Some(x) => span.merge(&x.span)
        };

        let kind = StatementKind::If { cond, then_stmt: Box::new(then_stmt), else_stmt: else_stmt.map(Box::new) };
        Statement::new(kind, span).into()
    }

    pub fn make_switch(switch_token: Token, cond: Box<Expression>, body: Statement) -> SemanticValue {
        let span = switch_token.span.merge(&body.span);
        let kind = StatementKind::Switch { cond, body: Box::new(body) };
        Statement::new(kind, span).into()
    }
    pub fn make_while(while_token: Token, cond: Box<Expression>, body: Statement, rparen: Option<Token>) -> SemanticValue {
        let span = while_token.span.merge(&body.span);
        let span = match rparen {
            None => span,
            Some(x) => x.span.merge(&span)
        };
        let kind = match while_token.kind {
            TokenKind::KeywordWhile => StatementKind::While { cond, body: Box::new(body) },
            TokenKind::KeywordDo => StatementKind::DoWhile { cond, body: Box::new(body) },
            _ => unreachable!()
        };
        Statement::new(kind, span).into()
    }

    pub fn make_for(for_token: Token, init: Option<Box<Expression>>, cond: Option<Box<Expression>>, step: Option<Box<Expression>>, body: Statement) -> SemanticValue {
        let span = for_token.span.merge(&body.span);
        let kind = StatementKind::For { init, cond, step, body: Box::new(body) };
        Statement::new(kind, span).into()
    }

    /// 第一个token是goto
    pub fn make_goto(goto: Token, label: Token) -> SemanticValue {
        let goto_span = goto.span;
        let label_span = label.span;

        let span = goto_span.merge(&label_span);
        let label = label.value.into_string().unwrap();
        let kind = StatementKind::Goto { label };
        Statement::new(kind, span).into()
    }


    pub fn make_continue_break(token: Token) -> SemanticValue {
        let span = token.span;
        let kind = match token.kind {
            TokenKind::KeywordContinue => StatementKind::Continue,
            TokenKind::KeywordBreak => StatementKind::Break,
            _ => unreachable!()
        };

        Statement::new(kind, span).into()
    }

    /// 第一个token是return
    pub fn make_return(ret: Token, expr: Option<Box<Expression>>) -> SemanticValue {
        let ret_span = ret.span;
        let span = match &expr {
            None => ret_span,
            Some(expr) => ret_span.merge(&expr.span)
        };

        let kind = StatementKind::Return(expr);
        Statement::new(kind, span).into()
    }

    pub fn make_compound(lbrace: Token, list: Option<BlockItemList>, rbrace: Token) -> SemanticValue {
        let span = lbrace.span.merge(&rbrace.span);
        let kind = StatementKind::Compound {lbrace: lbrace.span, list, rbrace: span};
        Statement::new(kind, span).into()
    }

}

impl Expression {

    pub fn make_literal(constant: Constant) -> SemanticValue {
        let span = constant.span;
        let kind = ExpressionKind::Literal(constant);
        Box::new(Expression::new(kind, None, span)).into()
    }

    pub fn make_id(token: Token) -> SemanticValue {
        let span = token.span;
        let name = token.value.into_string().unwrap();
        let kind = ExpressionKind::Id { name, decl_ref: None };
        Box::new(Expression::new(kind, None, span)).into()
    }

    /// 最后的token是 arr[...] <-这个字符，用来精确确定位置
    pub fn make_array_access(base: Box<Expression>, index: Box<Expression>, token: Token) -> SemanticValue {
        let span = base.span.merge(&token.span);
        let kind = ExpressionKind::ArrayAccess { base, index };
        Box::new(Expression::new(kind, None, span)).into()
    }

    /// 最后的token是 foo(...) <-这个字符，用来精确确定位置
    pub fn make_call(func: Box<Expression>, args: Vec<Box<Expression>>, token: Token) -> SemanticValue {
        let span = func.span.merge(&token.span);
        let kind = ExpressionKind::Call {func, args};

        Box::new(Expression::new(kind, None, span)).into()
    }

    pub fn make_field_access(base: Box<Expression>, field: Token) -> SemanticValue {
        let span = base.span.merge(&field.span);
        let field = field.value.into_string().unwrap();
        let kind = ExpressionKind::FieldAccess { base, field };

        Box::new(Expression::new(kind, None, span)).into()
    }

    pub fn make_arrow(base: Box<Expression>, field: Token) -> SemanticValue {
        let span = base.span.merge(&field.span);
        let field = field.value.into_string().unwrap();
        let kind = ExpressionKind::Arrow { base, field };

        Box::new(Expression::new(kind, None, span)).into()

    }

    ///
    /// 构建 前后置 ++ --
    /// # Arguments
    /// exprs:
    /// token:
    /// post: 是否是后置
    ///
    pub fn make_update(expr: Box<Expression>, token: Token, post: bool) -> SemanticValue {
        let span = token.span.merge(&expr.span);

        let kind = match (token.kind, post) {
            (TokenKind::OpDec, true) => ExpressionKind::PostDec,
            (TokenKind::OpDec, false) => ExpressionKind::PreDec,
            (TokenKind::OpInc, true) => ExpressionKind::PostInc,
            (TokenKind::OpInc, false) => ExpressionKind::PreInc,
            _ => unreachable!()
        };

        let kind = kind(expr);
        Box::new(Expression { kind, ty: None, span }).into()
    }

    pub fn make_unary(token: Token, expr: Box<Expression>) -> SemanticValue {
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


        let expr_kind = ExpressionKind::Unary {op, expr};

        Box::new(Expression::new(expr_kind, None, span)).into()
    }

    /// 第一个token是sizeof的值 -> sizeof exprs
    pub fn make_sizeof_expr(_sizeof: Token, expr: Box<Expression>) -> SemanticValue {
        let span = expr.span;
        let kind = ExpressionKind::SizeofExpr(expr);

        Box::new(Expression::new(kind, None, span)).into()
    }

    /// 第一个token是sizeof的值 -> sizeof(type) <- 第二个是第二个括号
    pub fn make_sizeof_type(kw_sizeof: Token, rparen: Token, typ: Type, lparen: Token) -> SemanticValue {
        todo!()
    }


    /// 第一个token 是类型转换的第一个括号-> (X)X
    pub fn make_cast(token: Token, typ: Type, expr: Box<Expression>) -> SemanticValue {
        todo!();
       
    }

    pub fn make_binary(lhs: Box<Expression>, token: Token, rhs: Box<Expression>) -> SemanticValue {
        let span = lhs.span.merge(&rhs.span);
        let span_token = token.span;


        let op_kind: BinaryOpKind = token.try_into().unwrap();
        let op = BinaryOp::new(op_kind, span_token);

        let kind = ExpressionKind::Binary { lhs, op, rhs };

        Box::new(Expression::new(kind, None, span)).into()
    }


    pub fn make_conditional(cond: Box<Expression>, then_expr: Box<Expression>, else_expr: Box<Expression>) -> SemanticValue {
        let span = cond.span.merge(&else_expr.span);
        let kind = ExpressionKind::Conditional {
            cond,
            then_expr,
            else_expr
        };

        Box::new(Expression::new(kind, None, span)).into()
    }

    pub fn make_assign(lhs: Box<Expression>, token: Token, rhs: Box<Expression>) -> SemanticValue {
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

    pub fn make_comma(mut exprs: Vec<Box<Expression>>, expr: Box<Expression>) -> SemanticValue {
        exprs.push(expr);
        exprs.into()
    }

}

impl Constant {
    pub fn make(token: Token) -> SemanticValue {
        Constant::try_from(token).unwrap().into()
    }

    pub fn insert_str(mut constant: Constant, token: Token) -> SemanticValue {
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
impl Initializer {
    pub fn make_expr(expr: Box<nodes::Expression>) -> SemanticValue {
        Self::Scalar(expr).into()
    }

    pub fn make_init_list(lbrace: Token, mut list: InitializerList, comma: Option<Token>, rbrace: Token) -> SemanticValue {
        if let Some(comma) = comma {
            list.push_sep(comma.span);

        }
        Self::List {
            lbrace: lbrace.span,
            list,
            rbrace: rbrace.span
        }.into()
    }


    pub fn make_list(init: Self) -> SemanticValue {
        InitializerList::new(init).into()
    }

    pub fn push(mut list: InitializerList, comma: Token, init: Self) -> SemanticValue {
        list.push(comma.span, init);
        list.into()
    }



}



impl Decl {

    pub fn make(decl_spec: DeclSpec, init_decl_list: Option<InitDeclList>, semi: Token) -> SemanticValue {
        let span = semi.span.merge(&decl_spec.span);
        let init_decl_list = init_decl_list.unwrap_or_default();

        

        for x in init_decl_list.list {
            let mut chunks = Vec::new();
            let name = flat_chunks(x.decl, &mut chunks);
            
        }
        




        todo!()
    }

    pub fn make_list(decl: Box<Decl>) -> SemanticValue {
        DeclList::from([decl]).into()
    }

    pub fn push(mut decl_list: DeclList, decl: Box<Decl>) -> SemanticValue {
        decl_list.push(decl);
        decl_list.into()
    }

}